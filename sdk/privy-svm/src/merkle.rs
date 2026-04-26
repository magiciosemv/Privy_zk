use std::collections::BTreeMap;

use crate::hash::poseidon_hash;
use crate::types::MerkleProof;

pub fn compute_merkle_root(
    leaf_hash: &[u8; 32],
    leaf_index: u64,
    siblings: &[[u8; 32]],
) -> [u8; 32] {
    let mut current = *leaf_hash;
    for (i, sibling) in siblings.iter().enumerate() {
        let bit = (leaf_index >> i) & 1;
        if bit == 0 {
            current = poseidon_hash(&current, sibling);
        } else {
            current = poseidon_hash(sibling, &current);
        }
    }
    current
}

pub fn verify_merkle_proof(
    leaf_hash: &[u8; 32],
    leaf_index: u64,
    siblings: &[[u8; 32]],
    root: &[u8; 32],
) -> bool {
    let computed = compute_merkle_root(leaf_hash, leaf_index, siblings);
    computed == *root
}

pub struct MerkleTree {
    depth: usize,
    nodes: BTreeMap<(usize, u64), [u8; 32]>,
    default_nodes: Vec<[u8; 32]>,
    next_index: u64,
    leaf_count: u64,
}

impl MerkleTree {
    pub fn new(depth: usize) -> Self {
        let mut default_nodes = Vec::with_capacity(depth + 1);
        let mut node = [0u8; 32];
        default_nodes.push(node);
        for _ in 0..depth {
            node = poseidon_hash(&node, &node);
            default_nodes.push(node);
        }
        Self {
            depth,
            nodes: BTreeMap::new(),
            default_nodes,
            next_index: 0,
            leaf_count: 0,
        }
    }

    pub fn insert(&mut self, hash: [u8; 32]) -> u64 {
        let index = self.next_index;
        self.nodes.insert((0, index), hash);

        let mut idx = index;
        for level in 1..=self.depth {
            let sibling_idx = idx ^ 1;
            let sibling = self
                .nodes
                .get(&(level - 1, sibling_idx))
                .copied()
                .unwrap_or(self.default_nodes[level - 1]);

            let parent = if idx & 1 == 0 {
                let current = self
                    .nodes
                    .get(&(level - 1, idx))
                    .copied()
                    .unwrap_or(self.default_nodes[level - 1]);
                poseidon_hash(&current, &sibling)
            } else {
                poseidon_hash(&sibling, &self
                    .nodes
                    .get(&(level - 1, idx))
                    .copied()
                    .unwrap_or(self.default_nodes[level - 1]))
            };

            let parent_idx = idx >> 1;
            self.nodes.insert((level, parent_idx), parent);
            idx = parent_idx;
        }

        self.next_index += 1;
        self.leaf_count += 1;
        index
    }

    pub fn get_proof(&self, leaf_index: u64) -> MerkleProof {
        let mut siblings = Vec::with_capacity(self.depth);
        let mut idx = leaf_index;
        for level in 0..self.depth {
            let sibling_idx = idx ^ 1;
            let sibling = self
                .nodes
                .get(&(level, sibling_idx))
                .copied()
                .unwrap_or(self.default_nodes[level]);
            siblings.push(sibling);
            idx >>= 1;
        }
        MerkleProof {
            leaf_index,
            siblings,
        }
    }

    pub fn root(&self) -> [u8; 32] {
        self.nodes
            .get(&(self.depth, 0))
            .copied()
            .unwrap_or(self.default_nodes[self.depth])
    }

    pub fn leaf_count(&self) -> u64 {
        self.leaf_count
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaf = [1u8; 32];
        let siblings: Vec<[u8; 32]> = (0..32)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i as u8;
                h
            })
            .collect();
        let root = compute_merkle_root(&leaf, 0, &siblings);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_verify_merkle_proof() {
        let leaf = [1u8; 32];
        let siblings: Vec<[u8; 32]> = (0..32)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i as u8;
                h
            })
            .collect();
        let root = compute_merkle_root(&leaf, 0, &siblings);
        assert!(verify_merkle_proof(&leaf, 0, &siblings, &root));
        assert!(!verify_merkle_proof(&leaf, 1, &siblings, &root));
    }

    #[test]
    fn test_merkle_tree_insert_and_proof() {
        let depth = 20;
        let mut tree = MerkleTree::new(depth);

        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let h3 = [3u8; 32];

        let idx1 = tree.insert(h1);
        let idx2 = tree.insert(h2);
        let idx3 = tree.insert(h3);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(tree.leaf_count(), 3);

        let root = tree.root();

        let proof1 = tree.get_proof(0);
        assert!(proof1.verify(&h1, &root));
        assert!(!proof1.verify(&h2, &root));

        let proof2 = tree.get_proof(1);
        assert!(proof2.verify(&h2, &root));

        let proof3 = tree.get_proof(2);
        assert!(proof3.verify(&h3, &root));
    }

    #[test]
    fn test_merkle_tree_root_changes() {
        let mut tree = MerkleTree::new(10);
        let root_empty = tree.root();

        let h = [42u8; 32];
        tree.insert(h);

        let root_with_leaf = tree.root();
        assert_ne!(root_empty, root_with_leaf);
    }

    #[test]
    fn test_merkle_tree_empty_proof() {
        let tree = MerkleTree::new(10);
        let root = tree.root();
        let proof = tree.get_proof(0);
        let empty_leaf = [0u8; 32];
        assert!(proof.verify(&empty_leaf, &root));
    }
}
