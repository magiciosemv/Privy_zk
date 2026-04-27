use anchor_lang::prelude::*;
use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::{PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use sha2::{Digest, Sha256};

declare_id!("BuiA3HBdZhbsHZDGGVvVeudVGxeMjg2n5uozTWEePTzN");

/// Maximum size of serialized verification key data in bytes.
/// Covers BN254 Groth16 VK with up to ~55 public inputs.
pub const MAX_VK_SIZE: usize = 2048;

/// Maximum size of a single batch verification call.
pub const MAX_BATCH_SIZE: usize = 20;

#[account]
pub struct VerifierState {
    pub admin: Pubkey,
    pub total_verifications: u64,
    pub circuit_count: u64,
}

impl VerifierState {
    pub const SPACE: usize = 8 + 32 + 8 + 8;
}

#[account]
pub struct VerificationKeyAccount {
    pub circuit_id: Pubkey,
    pub proof_type: u8,
    pub is_active: bool,
    pub vk_len: u32,
    pub vk_data: [u8; MAX_VK_SIZE],
}

impl VerificationKeyAccount {
    pub const BASE_SPACE: usize = 8 + 32 + 1 + 1 + 4 + MAX_VK_SIZE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchProofInput {
    pub circuit_id: Pubkey,
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<[u8; 32]>,
}

#[event]
pub struct ProofVerified {
    pub circuit_id: Pubkey,
    pub prover: Pubkey,
    pub proof_type: u8,
    pub public_inputs_hash: [u8; 32],
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid proof data")]
    InvalidProof,
    #[msg("Verification key not found for circuit")]
    VerificationKeyNotFound,
    #[msg("Invalid public inputs")]
    InvalidPublicInputs,
    #[msg("Proof verification failed")]
    ProofVerificationFailed,
    #[msg("Unauthorized: only admin can call this")]
    UnauthorizedAdmin,
    #[msg("Circuit already exists")]
    CircuitAlreadyExists,
    #[msg("Circuit is not active")]
    CircuitInactive,
    #[msg("Invalid verification key data")]
    InvalidVerificationKey,
    #[msg("Batch size exceeds maximum allowed")]
    BatchTooLarge,
    #[msg("Arithmetic overflow")]
    ArithmeticError,
}

// ---------------------------------------------------------------------------
//  Helper functions (must be before #[program] for macro visibility)
// ---------------------------------------------------------------------------

/// Core BN254 Groth16 verification logic shared by single and batch paths.
fn do_verify_groth16(
    vk_account: &VerificationKeyAccount,
    proof_data: &[u8],
    public_inputs: &[[u8; 32]],
) -> Result<()> {
    // Deserialize the verification key from on-chain storage.
    let vk_bytes = &vk_account.vk_data[..vk_account.vk_len as usize];
    let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_bytes)
        .map_err(|_| ErrorCode::InvalidVerificationKey)?;

    // Prepare the VK (pre-computes pairing contributions).
    let pvk = PreparedVerifyingKey::<Bn254>::from(vk);

    // Deserialize the proof.
    let proof = deserialize_proof(proof_data)?;

    // Convert public inputs to Fr field elements.
    let inputs: Vec<Fr> = public_inputs
        .iter()
        .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
        .collect();

    // Run the pairing-based verification.
    // ark-groth16 0.4 uses the stand-alone verify_proof function
    let verified = ark_groth16::Groth16::<Bn254>::verify_proof(&pvk, &proof, &inputs)
        .map_err(|_| ErrorCode::ProofVerificationFailed)?;

    require!(verified, ErrorCode::ProofVerificationFailed);

    Ok(())
}

/// Deserialize a `Proof<Bn254>` from raw bytes.
///
/// Supports both compressed (128 bytes) and uncompressed (256 bytes) encodings.
/// Compressed:  a=G1(32B)  b=G2(64B)  c=G1(32B)
/// Uncompressed: a=G1(64B)  b=G2(128B) c=G1(64B)
fn deserialize_proof(data: &[u8]) -> Result<Proof<Bn254>> {
    match data.len() {
        128 => Proof::<Bn254>::deserialize_compressed(data)
            .map_err(|_| ErrorCode::InvalidProof.into()),
        256 => Proof::<Bn254>::deserialize_uncompressed(data)
            .map_err(|_| ErrorCode::InvalidProof.into()),
        _ => Err(ErrorCode::InvalidProof.into()),
    }
}

/// SHA-256 hash of all public inputs, used to uniquely identify a proof instance
/// in emitted events without revealing the values themselves.
fn hash_public_inputs(inputs: &[[u8; 32]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input);
    }
    let digest = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

// ── Program ────────────────────────────────────────────────────────────────

#[program]
pub mod privy_verifier {
    use super::*;

    /// Initialize the verifier program state.
    /// Creates the global VerifierState PDA at seeds = [b"privy-verifier"].
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.verifier_state;
        state.admin = ctx.accounts.admin.key();
        state.total_verifications = 0;
        state.circuit_count = 0;
        Ok(())
    }

    /// Verify a single Groth16 ZK proof on-chain using BN254 pairings.
    ///
    /// * `circuit_id` - Identifies which circuit's verification key to use.
    /// * `proof_data`  - Serialized Proof<Bn254> (128 bytes compressed, 256 bytes uncompressed).
    /// * `public_inputs` - Array of 32-byte field-element public inputs.
    pub fn verify_groth16(
        ctx: Context<VerifyGroth16>,
        _circuit_id: Pubkey,
        proof_data: Vec<u8>,
        public_inputs: Vec<[u8; 32]>,
    ) -> Result<()> {
        let vk_account = &ctx.accounts.verification_key;

        require!(vk_account.is_active, ErrorCode::CircuitInactive);
        require!(vk_account.proof_type == 0, ErrorCode::InvalidProof);

        do_verify_groth16(vk_account, &proof_data, &public_inputs)?;

        let public_inputs_hash = hash_public_inputs(&public_inputs);
        let clock = Clock::get()?;

        let state = &mut ctx.accounts.verifier_state;
        state.total_verifications = state
            .total_verifications
            .checked_add(1)
            .ok_or(ErrorCode::ArithmeticError)?;

        emit!(ProofVerified {
            circuit_id: vk_account.circuit_id,
            prover: ctx.accounts.prover.key(),
            proof_type: vk_account.proof_type,
            public_inputs_hash,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Batch-verify multiple Groth16 proofs for the same circuit.
    /// Uses a single named VK account and verifies N proofs in one transaction.
    /// This is more efficient than N separate verify_groth16 calls.
    pub fn batch_verify(
        ctx: Context<BatchVerify>,
        _circuit_id: Pubkey,
        proofs: Vec<BatchProofInput>,
    ) -> Result<()> {
        require!(!proofs.is_empty(), ErrorCode::InvalidProof);
        require!(proofs.len() <= MAX_BATCH_SIZE, ErrorCode::BatchTooLarge);

        let vk = &ctx.accounts.verification_key;
        require!(vk.is_active, ErrorCode::CircuitInactive);
        require!(vk.proof_type == 0, ErrorCode::InvalidProof);

        let clock = Clock::get()?;
        let mut verified_count: u64 = 0;

        for proof_input in proofs.iter() {
            do_verify_groth16(vk, &proof_input.proof_data, &proof_input.public_inputs)?;

            let public_inputs_hash = hash_public_inputs(&proof_input.public_inputs);
            emit!(ProofVerified {
                circuit_id: vk.circuit_id,
                prover: ctx.accounts.prover.key(),
                proof_type: vk.proof_type,
                public_inputs_hash,
                timestamp: clock.unix_timestamp,
            });

            verified_count = verified_count
                .checked_add(1)
                .ok_or(ErrorCode::ArithmeticError)?;
        }

        let state = &mut ctx.accounts.verifier_state;
        state.total_verifications = state
            .total_verifications
            .checked_add(verified_count)
            .ok_or(ErrorCode::ArithmeticError)?;

        Ok(())
    }

    /// Store a verification key on-chain for a specific circuit.
    /// Admin-only. Creates a PDA at seeds = [b"vk", circuit_id].
    ///
    /// * `circuit_id` - Unique identifier for the circuit (typically a pubkey).
    /// * `proof_type` - 0 = Groth16, 1 = Plonk.
    /// * `vk_data`    - Canonical-serialized verification key bytes.
    pub fn create_verification_key(
        ctx: Context<CreateVerificationKey>,
        circuit_id: Pubkey,
        proof_type: u8,
        vk_data: Vec<u8>,
    ) -> Result<()> {
        require!(vk_data.len() <= MAX_VK_SIZE, ErrorCode::InvalidVerificationKey);
        require!(proof_type <= 1, ErrorCode::InvalidVerificationKey);

        // Validate VK data: attempt to deserialize so we fail early on garbage.
        if proof_type == 0 {
            VerifyingKey::<Bn254>::deserialize_compressed(vk_data.as_slice())
                .map_err(|_| ErrorCode::InvalidVerificationKey)?;
        }
        // Plonk VK validation would go here when supported.

        let vk_account = &mut ctx.accounts.verification_key;
        vk_account.circuit_id = circuit_id;
        vk_account.proof_type = proof_type;
        vk_account.is_active = true;
        vk_account.vk_len = vk_data.len() as u32;
        // Zero out the buffer before copying to avoid stale data.
        vk_account.vk_data.fill(0);
        vk_account.vk_data[..vk_data.len()].copy_from_slice(&vk_data);

        let state = &mut ctx.accounts.verifier_state;
        state.circuit_count = state
            .circuit_count
            .checked_add(1)
            .ok_or(ErrorCode::ArithmeticError)?;

        Ok(())
    }

    /// Transfer admin authority to a new pubkey.
    /// Only callable by the current admin.
    pub fn update_admin(ctx: Context<UpdateAdmin>, new_admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.verifier_state;
        state.admin = new_admin;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Account context structs
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = VerifierState::SPACE,
        seeds = [b"privy-verifier"],
        bump
    )]
    pub verifier_state: Account<'info, VerifierState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(circuit_id: Pubkey, proof_data: Vec<u8>, public_inputs: Vec<[u8; 32]>)]
pub struct VerifyGroth16<'info> {
    #[account(
        mut,
        seeds = [b"privy-verifier"],
        bump
    )]
    pub verifier_state: Account<'info, VerifierState>,

    #[account(
        seeds = [b"vk", circuit_id.as_ref()],
        bump
    )]
    pub verification_key: Account<'info, VerificationKeyAccount>,

    #[account(mut)]
    pub prover: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(circuit_id: Pubkey, proof_type: u8, vk_data: Vec<u8>)]
pub struct CreateVerificationKey<'info> {
    #[account(
        mut,
        seeds = [b"privy-verifier"],
        bump,
        has_one = admin @ ErrorCode::UnauthorizedAdmin
    )]
    pub verifier_state: Account<'info, VerifierState>,

    #[account(
        init,
        payer = admin,
        space = VerificationKeyAccount::BASE_SPACE,
        seeds = [b"vk", circuit_id.as_ref()],
        bump
    )]
    pub verification_key: Account<'info, VerificationKeyAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(
        mut,
        seeds = [b"privy-verifier"],
        bump,
        has_one = admin @ ErrorCode::UnauthorizedAdmin
    )]
    pub verifier_state: Account<'info, VerifierState>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(circuit_id: Pubkey)]
pub struct BatchVerify<'info> {
    #[account(
        mut,
        seeds = [b"privy-verifier"],
        bump
    )]
    pub verifier_state: Account<'info, VerifierState>,

    #[account(
        seeds = [b"vk", circuit_id.as_ref()],
        bump
    )]
    pub verification_key: Account<'info, VerificationKeyAccount>,

    #[account(mut)]
    pub prover: Signer<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{PrimeField, Zero};

    #[test]
    fn test_hash_public_inputs_deterministic() {
        let inputs = vec![[1u8; 32], [2u8; 32]];
        let h1 = hash_public_inputs(&inputs);
        let h2 = hash_public_inputs(&inputs);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_public_inputs_different() {
        let inputs1 = vec![[1u8; 32], [2u8; 32]];
        let inputs2 = vec![[2u8; 32], [1u8; 32]];
        assert_ne!(hash_public_inputs(&inputs1), hash_public_inputs(&inputs2));
    }

    #[test]
    fn test_hash_public_inputs_empty() {
        let inputs: Vec<[u8; 32]> = vec![];
        let hash = hash_public_inputs(&inputs);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_deserialize_proof_invalid_length() {
        let result = deserialize_proof(&[0u8; 100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::InvalidProof as u32, 6000);
    }

    #[test]
    fn test_verifier_state_space() {
        assert_eq!(VerifierState::SPACE, 8 + 32 + 8 + 8);
    }

    #[test]
    fn test_vk_account_base_space() {
        assert!(VerificationKeyAccount::BASE_SPACE >= 8 + 32 + 1 + 1 + 4 + MAX_VK_SIZE);
    }

    #[test]
    fn test_fr_from_be_bytes() {
        let bytes = [1u8; 32];
        let fr = Fr::from_be_bytes_mod_order(&bytes);
        assert!(!fr.is_zero());
    }

    #[test]
    fn test_max_batch_size_positive() {
        assert!(MAX_BATCH_SIZE > 0);
        assert!(MAX_BATCH_SIZE <= 100);
    }
}
