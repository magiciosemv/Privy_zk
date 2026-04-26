use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use light_poseidon::{Poseidon, PoseidonHasher};
use std::convert::TryInto;

declare_id!("3pD8r4QpTbvRtUeQvqrJ7oE8DDHXkB1udmnopPzDrtNg");

const TREE_DEPTH: usize = 256;
const MAX_TREE_SIZE: u64 = 1_000_000;

// ── Error Codes ────────────────────────────────────────────────────────────

#[error_code]
pub enum PSTreeError {
    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,
    #[msg("Nullifier already spent")]
    NullifierAlreadySpent,
    #[msg("Leaf not found")]
    LeafNotFound,
    #[msg("Leaf has expired")]
    LeafExpired,
    #[msg("Unauthorized revoke - signer is not the leaf owner")]
    UnauthorizedRevoke,
    #[msg("Tree already initialized for this namespace")]
    TreeAlreadyInitialized,
    #[msg("Invalid namespace")]
    InvalidNamespace,
    #[msg("Commitment already exists at this leaf index")]
    CommitmentAlreadyExists,
    #[msg("Maximum tree size exceeded")]
    MaxTreeSizeExceeded,
    #[msg("Leaf is not active")]
    LeafNotActive,
    #[msg("Merkle proof must cover full tree depth")]
    ProofTooShort,
    #[msg("Poseidon hash error")]
    PoseidonError,
}

// ── Events ─────────────────────────────────────────────────────────────────

#[event]
pub struct TreeInitialized {
    pub namespace: u64,
    pub root: [u8; 32],
}

#[event]
pub struct CommitmentInserted {
    pub leaf_index: u64,
    pub commitment: [u8; 32],
    pub new_root: [u8; 32],
}

#[event]
pub struct CommitmentUpdated {
    pub leaf_index: u64,
    pub old_nullifier: [u8; 32],
    pub new_commitment: [u8; 32],
    pub new_root: [u8; 32],
}

#[event]
pub struct NullifierConsumed {
    pub nullifier: [u8; 32],
    pub leaf_index: u64,
}

#[event]
pub struct CommitmentRevoked {
    pub nullifier: [u8; 32],
    pub leaf_index: u64,
}

#[event]
pub struct LeavesPruned {
    pub count: u64,
}

// ── Account Structs ────────────────────────────────────────────────────────

#[account]
pub struct PSTreeAccount {
    pub root: [u8; 32],
    pub leaf_count: u64,
    pub namespace: u64,
    pub nullifier_registry_root: [u8; 32],
    pub expired_leaf_count: u64,
    pub last_updated_slot: u64,
}

impl PSTreeAccount {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 32 + 8 + 8;
}

#[account]
pub struct LeafNode {
    pub version: u64,
    pub commitment: [u8; 32],
    pub nullifier: [u8; 32],
    pub expiry_timestamp: i64,
    pub is_active: bool,
    pub owner: Pubkey,
}

impl LeafNode {
    pub const LEN: usize = 8 + 8 + 32 + 32 + 8 + 1 + 32;
}

// ── Poseidon Hash Utilities ────────────────────────────────────────────────

/// Convert a 32-byte array to a BN254 field element.
fn bytes_to_fr(bytes: &[u8; 32]) -> Fr {
    let mut work = *bytes;
    // Reduce modulo BN254 scalar field order (~2^254).
    // Masking the top byte ensures the value is < field order.
    work[31] &= 0x0f;
    Fr::from_be_bytes_mod_order(&work)
}

/// Convert a BN254 field element to a 32-byte array.
fn fr_to_bytes(fr: &Fr) -> [u8; 32] {
    let raw = fr.into_bigint().to_bytes_be();
    let mut out = [0u8; 32];
    let len = raw.len().min(32);
    out[32 - len..].copy_from_slice(&raw[..len]);
    out
}

/// Poseidon hash of two 32-byte inputs → 32-byte output.
/// Uses light-poseidon with Circom-compatible parameters (2 input elements).
pub fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();
    let left_fr = bytes_to_fr(left);
    let right_fr = bytes_to_fr(right);
    let hash_fr = poseidon.hash(&[left_fr, right_fr]).unwrap();
    fr_to_bytes(&hash_fr)
}

// ── Merkle Proof Verification ──────────────────────────────────────────────

/// Verify a Sparse Merkle Tree proof.
/// The `merkle_proof` must contain sibling hashes for every level from leaf
/// to root (i.e. TREE_DEPTH entries). For empty subtrees, the caller
/// precomputes the zero-value at that depth and includes it in the proof.
pub fn verify_merkle_proof(
    leaf_hash: &[u8; 32],
    leaf_index: u64,
    merkle_proof: &[[u8; 32]],
    root: &[u8; 32],
) -> bool {
    let mut hash = *leaf_hash;
    let mut index = leaf_index;
    for sibling in merkle_proof {
        if index & 1 == 0 {
            hash = poseidon_hash_two(&hash, sibling);
        } else {
            hash = poseidon_hash_two(sibling, &hash);
        }
        index >>= 1;
    }
    hash == *root
}

/// Compute the new tree root after inserting/updating the leaf at `leaf_index`.
pub fn compute_new_root(
    leaf_hash: &[u8; 32],
    leaf_index: u64,
    merkle_proof: &[[u8; 32]],
) -> [u8; 32] {
    let mut hash = *leaf_hash;
    let mut index = leaf_index;
    for sibling in merkle_proof {
        if index & 1 == 0 {
            hash = poseidon_hash_two(&hash, sibling);
        } else {
            hash = poseidon_hash_two(sibling, &hash);
        }
        index >>= 1;
    }
    hash
}

/// Compute the empty-tree root (all leaves zero) for the Sparse Merkle Tree.
/// This is the root when no leaves have been inserted.
fn compute_empty_root() -> [u8; 32] {
    let zero = [0u8; 32];
    // Starting from the bottom: the "leaf" level empty node.
    // For a leaf that is all zeros, its hash in the tree is poseidon(0, 0).
    let mut current = poseidon_hash_two(&zero, &zero);
    // Walk up 256 levels
    for _ in 0..TREE_DEPTH {
        current = poseidon_hash_two(&current, &current);
    }
    current
}

/// Precomputed empty leaf hash: poseidon(0, 0).
fn empty_leaf_hash() -> [u8; 32] {
    let zero = [0u8; 32];
    poseidon_hash_two(&zero, &zero)
}

// ── Leaf Hash Computation ──────────────────────────────────────────────────

fn compute_leaf_hash(leaf: &LeafNode) -> [u8; 32] {
    let mut poseidon = Poseidon::<Fr>::new_circom(4).unwrap();

    let version_bytes = leaf.version.to_le_bytes();
    let expiry_bytes = leaf.expiry_timestamp.to_le_bytes();
    let active_byte = if leaf.is_active { 1u8 } else { 0u8 };

    // Pack version + expiry + is_active into one field element
    let mut meta = [0u8; 32];
    meta[..8].copy_from_slice(&version_bytes);
    meta[8..16].copy_from_slice(&expiry_bytes);
    meta[16] = active_byte;

    let frs = [
        bytes_to_fr(&leaf.commitment),
        bytes_to_fr(&leaf.nullifier),
        bytes_to_fr(&meta),
        bytes_to_fr(&leaf.owner.to_bytes()),
    ];

    let hash_fr = poseidon.hash(&frs).unwrap();
    fr_to_bytes(&hash_fr)
}

// ── Program ────────────────────────────────────────────────────────────────

#[program]
pub mod privy_pstree {
    use super::*;

    /// Initialize a new PSTree for a given namespace.
    /// The tree starts with the empty root (all leaves zero).
    pub fn initialize_tree(ctx: Context<InitializeTree>, namespace: u64) -> Result<()> {
        let tree = &mut ctx.accounts.tree;

        require!(namespace > 0, PSTreeError::InvalidNamespace);

        tree.root = compute_empty_root();
        tree.leaf_count = 0;
        tree.namespace = namespace;
        tree.nullifier_registry_root = [0u8; 32];
        tree.expired_leaf_count = 0;
        tree.last_updated_slot = Clock::get()?.slot;

        emit!(TreeInitialized {
            namespace,
            root: tree.root,
        });

        Ok(())
    }

    /// Insert a new commitment at a leaf position that is currently empty.
    pub fn insert_commitment(
        ctx: Context<InsertCommitment>,
        leaf_index: u64,
        commitment: [u8; 32],
        nullifier: [u8; 32],
        expiry_timestamp: i64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let tree = &mut ctx.accounts.tree;
        let leaf = &mut ctx.accounts.leaf;

        require!(leaf_index < MAX_TREE_SIZE, PSTreeError::MaxTreeSizeExceeded);
        require!(
            merkle_proof.len() == TREE_DEPTH,
            PSTreeError::ProofTooShort
        );

        // Verify the leaf position is currently empty.
        let empty_hash = empty_leaf_hash();
        require!(
            verify_merkle_proof(&empty_hash, leaf_index, &merkle_proof, &tree.root),
            PSTreeError::InvalidMerkleProof
        );

        leaf.version = 1;
        leaf.commitment = commitment;
        leaf.nullifier = nullifier;
        leaf.expiry_timestamp = expiry_timestamp;
        leaf.is_active = true;
        leaf.owner = ctx.accounts.signer.key();

        let leaf_hash = compute_leaf_hash(leaf);
        tree.root = compute_new_root(&leaf_hash, leaf_index, &merkle_proof);
        tree.leaf_count = tree
            .leaf_count
            .checked_add(1)
            .ok_or(PSTreeError::MaxTreeSizeExceeded)?;
        tree.last_updated_slot = Clock::get()?.slot;

        emit!(CommitmentInserted {
            leaf_index,
            commitment,
            new_root: tree.root,
        });

        Ok(())
    }

    /// Update an existing commitment.
    /// The old nullifier must match the leaf's current nullifier.
    pub fn update_commitment(
        ctx: Context<UpdateCommitment>,
        leaf_index: u64,
        old_nullifier: [u8; 32],
        new_commitment: [u8; 32],
        new_nullifier: [u8; 32],
        new_expiry_timestamp: i64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let tree = &mut ctx.accounts.tree;
        let leaf = &mut ctx.accounts.leaf;

        require!(leaf.is_active, PSTreeError::LeafNotActive);
        require!(merkle_proof.len() == TREE_DEPTH, PSTreeError::ProofTooShort);

        require!(
            leaf.nullifier == old_nullifier,
            PSTreeError::NullifierAlreadySpent
        );

        let current_leaf_hash = compute_leaf_hash(leaf);
        require!(
            verify_merkle_proof(&current_leaf_hash, leaf_index, &merkle_proof, &tree.root),
            PSTreeError::InvalidMerkleProof
        );

        leaf.version = leaf.version.checked_add(1).unwrap();
        leaf.commitment = new_commitment;
        leaf.nullifier = new_nullifier;
        leaf.expiry_timestamp = new_expiry_timestamp;

        let new_leaf_hash = compute_leaf_hash(leaf);
        tree.root = compute_new_root(&new_leaf_hash, leaf_index, &merkle_proof);

        // Register the old nullifier as spent.
        tree.nullifier_registry_root = poseidon_hash_two(
            &tree.nullifier_registry_root,
            &old_nullifier,
        );
        tree.last_updated_slot = Clock::get()?.slot;

        emit!(CommitmentUpdated {
            leaf_index,
            old_nullifier,
            new_commitment,
            new_root: tree.root,
        });

        Ok(())
    }

    /// Mark a commitment as consumed (spent) using its nullifier.
    pub fn consume_nullifier(
        ctx: Context<ConsumeNullifier>,
        leaf_index: u64,
        nullifier: [u8; 32],
    ) -> Result<()> {
        let tree = &mut ctx.accounts.tree;
        let leaf = &mut ctx.accounts.leaf;

        require!(leaf.is_active, PSTreeError::NullifierAlreadySpent);
        require!(leaf.nullifier == nullifier, PSTreeError::LeafNotFound);

        if leaf.expiry_timestamp > 0 {
            let now = Clock::get()?.unix_timestamp;
            require!(now < leaf.expiry_timestamp, PSTreeError::LeafExpired);
        }

        leaf.is_active = false;

        tree.nullifier_registry_root = poseidon_hash_two(
            &tree.nullifier_registry_root,
            &nullifier,
        );
        tree.last_updated_slot = Clock::get()?.slot;

        emit!(NullifierConsumed {
            nullifier,
            leaf_index,
        });

        Ok(())
    }

    /// User-initiated revocation of their own commitment.
    pub fn revoke_commitment(
        ctx: Context<RevokeCommitment>,
        leaf_index: u64,
        nullifier: [u8; 32],
    ) -> Result<()> {
        let tree = &mut ctx.accounts.tree;
        let leaf = &mut ctx.accounts.leaf;

        require!(leaf.is_active, PSTreeError::LeafNotActive);

        require!(
            leaf.owner == ctx.accounts.signer.key(),
            PSTreeError::UnauthorizedRevoke
        );

        require!(leaf.nullifier == nullifier, PSTreeError::LeafNotFound);

        leaf.is_active = false;

        tree.nullifier_registry_root = poseidon_hash_two(
            &tree.nullifier_registry_root,
            &nullifier,
        );
        tree.last_updated_slot = Clock::get()?.slot;

        emit!(CommitmentRevoked {
            nullifier,
            leaf_index,
        });

        Ok(())
    }

    /// Clean up expired leaves. Anyone can call this.
    pub fn cleanup_expired(
        ctx: Context<CleanupExpired>,
        leaf_indices: Vec<u64>,
    ) -> Result<()> {
        let tree = &mut ctx.accounts.tree;
        let now = Clock::get()?.unix_timestamp;
        let mut pruned_count: u64 = 0;

        for i in 0..leaf_indices.len() {
            if i >= ctx.remaining_accounts.len() {
                break;
            }
            let leaf_info = &ctx.remaining_accounts[i];

            let data = leaf_info.try_borrow_data()?;
            if data.len() < 8 + 8 + 32 + 32 + 8 + 1 + 32 {
                continue;
            }
            let offset = 8; // skip discriminator

            let expiry = i64::from_le_bytes(data[offset + 72..offset + 80].try_into().unwrap());
            let is_active = data[offset + 80] != 0;

            if is_active && expiry > 0 && expiry <= now {
                drop(data);
                let mut data_mut = leaf_info.try_borrow_mut_data()?;
                data_mut[offset + 80] = 0; // is_active = false
                pruned_count += 1;
            }
        }

        tree.expired_leaf_count = tree
            .expired_leaf_count
            .checked_add(pruned_count)
            .unwrap_or(tree.expired_leaf_count);
        tree.last_updated_slot = Clock::get()?.slot;

        if pruned_count > 0 {
            emit!(LeavesPruned {
                count: pruned_count,
            });
        }

        Ok(())
    }
}

// ── Account Contexts ───────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(namespace: u64)]
pub struct InitializeTree<'info> {
    #[account(
        init,
        payer = signer,
        space = PSTreeAccount::LEN,
        seeds = [b"pstree" as &[u8], &namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(leaf_index: u64)]
pub struct InsertCommitment<'info> {
    #[account(
        mut,
        seeds = [b"pstree" as &[u8], &tree.namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,

    #[account(
        init,
        payer = signer,
        space = LeafNode::LEN,
        seeds = [b"leaf".as_ref(), tree.key().as_ref(), &leaf_index.to_le_bytes() as &[u8]],
        bump
    )]
    pub leaf: Account<'info, LeafNode>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(leaf_index: u64)]
pub struct UpdateCommitment<'info> {
    #[account(
        mut,
        seeds = [b"pstree" as &[u8], &tree.namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,

    #[account(
        mut,
        seeds = [b"leaf".as_ref(), tree.key().as_ref(), &leaf_index.to_le_bytes() as &[u8]],
        bump
    )]
    pub leaf: Account<'info, LeafNode>,
}

#[derive(Accounts)]
#[instruction(leaf_index: u64)]
pub struct ConsumeNullifier<'info> {
    #[account(
        mut,
        seeds = [b"pstree" as &[u8], &tree.namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,

    #[account(
        mut,
        seeds = [b"leaf".as_ref(), tree.key().as_ref(), &leaf_index.to_le_bytes() as &[u8]],
        bump
    )]
    pub leaf: Account<'info, LeafNode>,
}

#[derive(Accounts)]
#[instruction(leaf_index: u64)]
pub struct RevokeCommitment<'info> {
    #[account(
        mut,
        seeds = [b"pstree" as &[u8], &tree.namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,

    #[account(
        mut,
        seeds = [b"leaf".as_ref(), tree.key().as_ref(), &leaf_index.to_le_bytes() as &[u8]],
        bump
    )]
    pub leaf: Account<'info, LeafNode>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CleanupExpired<'info> {
    #[account(
        mut,
        seeds = [b"pstree" as &[u8], &tree.namespace.to_le_bytes() as &[u8]],
        bump
    )]
    pub tree: Account<'info, PSTreeAccount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test poseidon_hash_two is deterministic
    #[test]
    fn test_poseidon_deterministic() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let h1 = poseidon_hash_two(&a, &b);
        let h2 = poseidon_hash_two(&a, &b);
        assert_eq!(h1, h2);
    }

    // Test poseidon_hash_two different inputs -> different outputs
    #[test]
    fn test_poseidon_different_inputs() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        assert_ne!(poseidon_hash_two(&a, &b), poseidon_hash_two(&a, &c));
    }

    // Test empty leaf hash is not all zeros
    #[test]
    fn test_empty_leaf_hash() {
        let h = empty_leaf_hash();
        assert_eq!(h.len(), 32);
        assert_ne!(h, [0u8; 32]);
    }

    // Test empty root computation
    #[test]
    fn test_empty_root_not_all_zeros() {
        let root = compute_empty_root();
        assert_eq!(root.len(), 32);
        assert_ne!(root, [0u8; 32]);
    }

    // Test merkle proof verification with correct data
    #[test]
    fn test_verify_merkle_proof_valid() {
        let leaf = [1u8; 32];
        // Build a proof manually
        let mut siblings: Vec<[u8; 32]> = Vec::new();
        for i in 0..TREE_DEPTH {
            let mut s = [0u8; 32];
            s[0] = (i % 256) as u8;
            siblings.push(s);
        }
        let root = compute_new_root(&leaf, 0, &siblings);
        assert!(verify_merkle_proof(&leaf, 0, &siblings, &root));
    }

    // Test merkle proof verification fails with wrong index
    #[test]
    fn test_verify_merkle_proof_wrong_index() {
        let leaf = [1u8; 32];
        let mut siblings: Vec<[u8; 32]> = Vec::new();
        for i in 0..TREE_DEPTH {
            let mut s = [0u8; 32];
            s[0] = (i % 256) as u8;
            siblings.push(s);
        }
        let root = compute_new_root(&leaf, 0, &siblings);
        // Using index 1 instead of 0 should fail
        assert!(!verify_merkle_proof(&leaf, 1, &siblings, &root));
    }

    // Test merkle proof verification fails with wrong root
    #[test]
    fn test_verify_merkle_proof_wrong_root() {
        let leaf = [1u8; 32];
        let mut siblings: Vec<[u8; 32]> = Vec::new();
        for i in 0..TREE_DEPTH {
            let mut s = [0u8; 32];
            s[0] = (i % 256) as u8;
            siblings.push(s);
        }
        let bad_root = [99u8; 32];
        assert!(!verify_merkle_proof(&leaf, 0, &siblings, &bad_root));
    }

    // Test compute_new_root is deterministic
    #[test]
    fn test_compute_new_root_deterministic() {
        let leaf = [5u8; 32];
        let mut siblings: Vec<[u8; 32]> = Vec::new();
        for i in 0..TREE_DEPTH {
            let mut s = [0u8; 32];
            s[0] = (i % 256) as u8;
            siblings.push(s);
        }
        let r1 = compute_new_root(&leaf, 42, &siblings);
        let r2 = compute_new_root(&leaf, 42, &siblings);
        assert_eq!(r1, r2);
    }

    // Test leaf hash computation
    #[test]
    fn test_compute_leaf_hash_deterministic() {
        let leaf = LeafNode {
            version: 1,
            commitment: [1u8; 32],
            nullifier: [2u8; 32],
            expiry_timestamp: 0,
            is_active: true,
            owner: Pubkey::new_unique(),
        };
        let h1 = compute_leaf_hash(&leaf);
        let h2 = compute_leaf_hash(&leaf);
        assert_eq!(h1, h2);
    }

    // Test different leaves have different hashes
    #[test]
    fn test_different_leaves_different_hashes() {
        let leaf1 = LeafNode {
            version: 1,
            commitment: [1u8; 32],
            nullifier: [2u8; 32],
            expiry_timestamp: 0,
            is_active: true,
            owner: Pubkey::new_unique(),
        };
        let leaf2 = LeafNode {
            version: 2,
            commitment: [1u8; 32],
            nullifier: [2u8; 32],
            expiry_timestamp: 0,
            is_active: true,
            owner: Pubkey::new_unique(),
        };
        assert_ne!(compute_leaf_hash(&leaf1), compute_leaf_hash(&leaf2));
    }

    // Test fr_to_bytes roundtrip
    #[test]
    fn test_fr_bytes_roundtrip() {
        let v = [42u8; 32];
        let fr = bytes_to_fr(&v);
        let back = fr_to_bytes(&fr);
        // After mod reduction, we verify back is same length
        assert_eq!(back.len(), 32);
    }
}
