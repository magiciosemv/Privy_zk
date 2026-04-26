use borsh::BorshSerialize;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;

use crate::types::{MerkleProof, Nullifier, ProofData};

pub struct PrivyClient {
    pub rpc_url: String,
    pub verifier_program_id: Pubkey,
    pub pstree_program_id: Pubkey,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyError {
    #[error("RPC error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    #[error("Borsh serialization error: {0}")]
    BorshError(#[from] std::io::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

type Result<T> = std::result::Result<T, PrivyError>;

#[derive(BorshSerialize)]
struct VerifyProofIxData {
    proof_type: u8,
    proof_data: Vec<u8>,
    public_inputs: Vec<[u8; 32]>,
}

#[derive(BorshSerialize)]
struct InsertIxData {
    commitment: [u8; 32],
    leaf_index: u64,
    siblings: Vec<[u8; 32]>,
    namespace: u64,
}

#[derive(BorshSerialize)]
struct ConsumeIxData {
    nullifier: [u8; 32],
}

#[derive(BorshSerialize)]
struct RevokeIxData {
    nullifier: [u8; 32],
    authority: Pubkey,
}

impl PrivyClient {
    pub fn new(rpc_url: &str, verifier_id: Pubkey, pstree_id: Pubkey) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            verifier_program_id: verifier_id,
            pstree_program_id: pstree_id,
        }
    }

    pub fn build_verify_ix(&self, proof: &ProofData) -> Result<Instruction> {
        let ix_data = VerifyProofIxData {
            proof_type: proof.proof_type,
            proof_data: proof.proof_bytes.clone(),
            public_inputs: proof.public_inputs.clone(),
        };

        let mut buf = vec![0u8];
        ix_data.serialize(&mut buf)?;

        let (pstree_pda, _bump) = Pubkey::find_program_address(
            &[b"pstree", &0u64.to_le_bytes()],
            &self.pstree_program_id,
        );

        let accounts = vec![
            AccountMeta::new_readonly(pstree_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];

        Ok(Instruction {
            program_id: self.verifier_program_id,
            accounts,
            data: buf,
        })
    }

    pub fn build_insert_ix(
        &self,
        commitment: &[u8; 32],
        proof: &MerkleProof,
        namespace: u64,
    ) -> Result<Instruction> {
        let ix_data = InsertIxData {
            commitment: *commitment,
            leaf_index: proof.leaf_index,
            siblings: proof.siblings.clone(),
            namespace,
        };

        let mut buf = vec![0u8];
        ix_data.serialize(&mut buf)?;

        let (pstree_pda, _bump) = Pubkey::find_program_address(
            &[b"pstree", &namespace.to_le_bytes()],
            &self.pstree_program_id,
        );

        let accounts = vec![
            AccountMeta::new(pstree_pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];

        Ok(Instruction {
            program_id: self.pstree_program_id,
            accounts,
            data: buf,
        })
    }

    pub fn build_consume_ix(&self, nullifier: &Nullifier) -> Result<Instruction> {
        let ix_data = ConsumeIxData {
            nullifier: nullifier.bytes,
        };

        let mut buf = vec![2u8];
        ix_data.serialize(&mut buf)?;

        let (pstree_pda, _bump) = Pubkey::find_program_address(
            &[b"pstree", &0u64.to_le_bytes()],
            &self.pstree_program_id,
        );

        let accounts = vec![
            AccountMeta::new(pstree_pda, false),
        ];

        Ok(Instruction {
            program_id: self.pstree_program_id,
            accounts,
            data: buf,
        })
    }

    pub fn build_revoke_ix(
        &self,
        nullifier: &Nullifier,
        authority: &Pubkey,
    ) -> Result<Instruction> {
        let ix_data = RevokeIxData {
            nullifier: nullifier.bytes,
            authority: *authority,
        };

        let mut buf = vec![3u8];
        ix_data.serialize(&mut buf)?;

        let (pstree_pda, _bump) = Pubkey::find_program_address(
            &[b"pstree", &0u64.to_le_bytes()],
            &self.pstree_program_id,
        );

        let accounts = vec![
            AccountMeta::new(pstree_pda, false),
            AccountMeta::new_readonly(*authority, true),
        ];

        Ok(Instruction {
            program_id: self.pstree_program_id,
            accounts,
            data: buf,
        })
    }

    pub fn sign_and_send(
        &self,
        payer: &Keypair,
        instructions: &[Instruction],
    ) -> Result<Signature> {
        let rpc_client = RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .map_err(PrivyError::RpcError)?;

        let message = solana_sdk::message::Message::new_with_blockhash(
            instructions,
            Some(&payer.pubkey()),
            &recent_blockhash,
        );

        let mut tx = Transaction::new_unsigned(message);
        tx.sign(&[payer], recent_blockhash);

        rpc_client
            .send_and_confirm_transaction(&tx)
            .map_err(PrivyError::RpcError)
    }
}
