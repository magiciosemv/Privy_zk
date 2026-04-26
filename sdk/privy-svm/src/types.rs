use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofData {
    pub proof_type: u8,
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CommitmentData {
    pub value: [u8; 32],
    pub blinding_factor: [u8; 32],
    pub program_id: Pubkey,
    pub namespace: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Nullifier {
    pub bytes: [u8; 32],
}

impl Nullifier {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self { bytes: arr })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MerkleProof {
    pub leaf_index: u64,
    pub siblings: Vec<[u8; 32]>,
}

impl MerkleProof {
    pub fn root(&self, leaf_hash: &[u8; 32]) -> [u8; 32] {
        crate::merkle::compute_merkle_root(leaf_hash, self.leaf_index, &self.siblings)
    }

    pub fn verify(&self, leaf_hash: &[u8; 32], root: &[u8; 32]) -> bool {
        crate::merkle::verify_merkle_proof(leaf_hash, self.leaf_index, &self.siblings, root)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProofType {
    Groth16 = 0,
    Plonk = 1,
}

impl ProofType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Groth16),
            1 => Some(Self::Plonk),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectiveDisclosure {
    pub disclosed_attributes: Vec<u8>,
    pub disclosed_values: Vec<[u8; 32]>,
}

impl SelectiveDisclosure {
    pub fn new(disclosed_attributes: Vec<u8>, disclosed_values: Vec<[u8; 32]>) -> Self {
        Self {
            disclosed_attributes,
            disclosed_values,
        }
    }

    pub fn is_disclosed(&self, attr_index: usize) -> bool {
        let byte_idx = attr_index / 8;
        let bit_idx = attr_index % 8;
        byte_idx < self.disclosed_attributes.len()
            && (self.disclosed_attributes[byte_idx] & (1 << bit_idx)) != 0
    }
}
