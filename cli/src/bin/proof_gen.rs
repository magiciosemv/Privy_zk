/// Generate empty-tree merkle proof values for TypeScript test consumption
use privy_svm::hash::{poseidon_hash as sdk_poseidon};
use privy_svm::merkle::MerkleTree;

fn main() {
    let depth = 20;
    let tree = MerkleTree::new(depth);
    
    println!("Empty tree root (hex):");
    println!("{}", hex::encode(tree.root()));
    
    println!("\nProof for leaf_index 0 (empty tree siblings):");
    let proof = tree.get_proof(0);
    for (i, sibling) in proof.siblings.iter().enumerate() {
        println!("  Level {}: {}", i, hex::encode(*sibling));
    }
    
    // Insert a commitment and show root after
    let mut tree2 = MerkleTree::new(depth);
    let commitment = [0x42u8; 32];
    tree2.insert(commitment);
    println!("\nRoot after inserting commitment 0x42... at leaf 0:");
    println!("{}", hex::encode(tree2.root()));
    
    let proof_after = tree2.get_proof(1);
    println!("\nProof for leaf_index 1 after insert:");
    for (i, sibling) in proof_after.siblings.iter().enumerate() {
        println!("  Level {}: {}", i, hex::encode(*sibling));
    }
}
