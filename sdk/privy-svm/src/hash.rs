use sha2::{Digest, Sha256};
use solana_sdk::pubkey::Pubkey;

pub fn poseidon_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"poseidon_sim_");
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

pub fn hash_single(preimage: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(preimage);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

pub fn compute_commitment(
    value: &[u8; 32],
    blinding: &[u8; 32],
    program_id: &Pubkey,
    namespace: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"commitment");
    hasher.update(value);
    hasher.update(blinding);
    hasher.update(&program_id.to_bytes());
    hasher.update(&namespace.to_le_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

pub fn compute_nullifier(
    private_key: &[u8; 32],
    leaf_index: u64,
    version: u64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"nullifier");
    hasher.update(private_key);
    hasher.update(&leaf_index.to_le_bytes());
    hasher.update(&version.to_le_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

pub fn compute_leaf_hash(
    commitment: &[u8; 32],
    nullifier: &[u8; 32],
    version: u64,
    expiry: i64,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"leaf");
    hasher.update(commitment);
    hasher.update(nullifier);
    hasher.update(&version.to_le_bytes());
    hasher.update(&expiry.to_le_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_deterministic() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let h1 = poseidon_hash(&a, &b);
        let h2 = poseidon_hash(&a, &b);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_poseidon_different_inputs() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        assert_ne!(poseidon_hash(&a, &b), poseidon_hash(&a, &c));
        assert_ne!(poseidon_hash(&a, &b), poseidon_hash(&c, &b));
    }

    #[test]
    fn test_compute_commitment() {
        let value = [1u8; 32];
        let blinding = [2u8; 32];
        let program_id = Pubkey::new_unique();
        let c1 = compute_commitment(&value, &blinding, &program_id, 0);
        let c2 = compute_commitment(&value, &blinding, &program_id, 0);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_compute_nullifier() {
        let key = [1u8; 32];
        let n1 = compute_nullifier(&key, 0, 0);
        let n2 = compute_nullifier(&key, 0, 0);
        assert_eq!(n1, n2);
        let n3 = compute_nullifier(&key, 0, 1);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_compute_leaf_hash() {
        let commitment = [1u8; 32];
        let nullifier = [2u8; 32];
        let h1 = compute_leaf_hash(&commitment, &nullifier, 0, 0);
        let h2 = compute_leaf_hash(&commitment, &nullifier, 0, 0);
        assert_eq!(h1, h2);
    }
}
