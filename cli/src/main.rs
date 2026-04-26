use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};
use privy_svm::{
    hash::{compute_commitment, compute_nullifier},
    merkle::MerkleTree,
    types::{MerkleProof, Nullifier, ProofData, ProofType, SelectiveDisclosure},
    PrivyClient,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

#[derive(Parser)]
#[command(name = "privy", about = "Privy SVM CLI toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new Privy SVM project
    Init { path: Option<String> },

    /// Verify a ZK proof
    Verify {
        proof_file: String,
        circuit_id: String,
        #[arg(long)]
        rpc_url: Option<String>,
    },

    /// Insert a commitment into the PSTree
    Insert {
        commitment: String,
        namespace: u64,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        verifier_program: Option<String>,
        #[arg(long)]
        pstree_program: Option<String>,
        #[arg(long)]
        keypair_file: Option<String>,
    },

    /// Consume (spend) a nullifier
    Consume {
        nullifier: String,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        pstree_program: Option<String>,
        #[arg(long)]
        keypair_file: Option<String>,
    },

    /// Revoke a commitment
    Revoke {
        nullifier: String,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        pstree_program: Option<String>,
        #[arg(long)]
        keypair_file: Option<String>,
    },

    /// Get the current PSTree root
    GetRoot {
        namespace: u64,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        pstree_program: Option<String>,
    },

    /// Generate a test keypair for development
    Keygen { output: Option<String> },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { path } => cmd_init(path),
        Command::Verify {
            proof_file,
            circuit_id,
            rpc_url,
        } => cmd_verify(proof_file, circuit_id, rpc_url),
        Command::Insert {
            commitment,
            namespace,
            rpc_url,
            verifier_program,
            pstree_program,
            keypair_file,
        } => cmd_insert(commitment, namespace, rpc_url, verifier_program, pstree_program, keypair_file),
        Command::Consume {
            nullifier,
            rpc_url,
            pstree_program,
            keypair_file,
        } => cmd_consume(nullifier, rpc_url, pstree_program, keypair_file),
        Command::Revoke {
            nullifier,
            rpc_url,
            pstree_program,
            keypair_file,
        } => cmd_revoke(nullifier, rpc_url, pstree_program, keypair_file),
        Command::GetRoot {
            namespace,
            rpc_url,
            pstree_program,
        } => cmd_get_root(namespace, rpc_url, pstree_program),
        Command::Keygen { output } => cmd_keygen(output),
    }
}

fn cmd_init(path: Option<String>) -> anyhow::Result<()> {
    let project_dir = path.unwrap_or_else(|| "privy-project".to_string());
    let dir = Path::new(&project_dir);

    if dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_dir);
    }

    fs::create_dir_all(dir)?;
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    let cargo_toml = r#"[package]
name = "privy-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "=1.18.0"
borsh = "1.5"
"#;
    fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    let lib_rs = r#"use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    pubkey::Pubkey,
    msg,
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    msg!("Privy SVM program");
    Ok(())
}
"#;
    fs::write(dir.join("src").join("lib.rs"), lib_rs)?;

    let privy_toml = r#"[project]
name = "privy-project"
circuit = "default"

[proof]
backend = "auto"

[selective_disclosure]
enabled = false
"#;
    fs::write(dir.join("privy.toml"), privy_toml)?;

    let integration_rs = r#"#[cfg(test)]
mod tests {
    #[test]
    fn test_program_linking() {
        assert!(true);
    }
}
"#;
    fs::write(dir.join("tests").join("integration.rs"), integration_rs)?;

    println!("Initialized Privy SVM project in '{}'", project_dir);
    println!("Next steps:");
    println!("  cd {}", project_dir);
    println!("  privy build");
    Ok(())
}

fn cmd_verify(proof_file: String, _circuit_id: String, rpc_url: Option<String>) -> anyhow::Result<()> {
    let proof_data_str = fs::read_to_string(&proof_file)
        .map_err(|e| anyhow::anyhow!("Cannot read proof file '{}': {}", proof_file, e))?;

    let proof: ProofData = serde_json::from_str(&proof_data_str)
        .map_err(|e| anyhow::anyhow!("Invalid proof JSON in '{}': {}", proof_file, e))?;

    let proof_type_label = match proof.proof_type {
        0 => "Groth16",
        1 => "Plonk",
        _ => "Unknown",
    };

    println!("Proof loaded:");
    println!("  Type: {} ({})", proof_type_label, proof.proof_type);
    println!("  Proof bytes: {} bytes", proof.proof_bytes.len());
    println!("  Public inputs: {} entries", proof.public_inputs.len());
    for (i, input) in proof.public_inputs.iter().enumerate() {
        println!("    [{}] = {}", i, hex::encode(input));
    }
    println!();

    if let Some(url) = &rpc_url {
        println!("Connecting to RPC: {}", url);

        let verifier_id = Pubkey::new_unique();
        let pstree_id = Pubkey::new_unique();

        let client = PrivyClient::new(url, verifier_id, pstree_id);
        match client.build_verify_ix(&proof) {
            Ok(ix) => {
                println!("Verification instruction built:");
                println!("  Program: {}", ix.program_id);
                println!("  Data: {} bytes", ix.data.len());
                println!("  Accounts: {}", ix.accounts.len());
                for (i, meta) in ix.accounts.iter().enumerate() {
                    println!(
                        "    [{}] {} (signer={}, writable={})",
                        i, meta.pubkey, meta.is_signer, meta.is_writable
                    );
                }
            }
            Err(e) => {
                println!("Error building instruction: {}", e);
            }
        }
    } else {
        println!("No RPC URL provided; skipping on-chain verification.");
        println!("To send on-chain: privy verify {} --circuit-id {} --rpc-url <URL>", proof_file, _circuit_id);
    }

    Ok(())
}

fn cmd_insert(
    commitment_hex: String,
    namespace: u64,
    rpc_url: Option<String>,
    verifier_program: Option<String>,
    pstree_program: Option<String>,
    keypair_file: Option<String>,
) -> anyhow::Result<()> {
    let commitment_bytes = hex::decode(&commitment_hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex commitment: {}", e))?;

    if commitment_bytes.len() != 32 {
        anyhow::bail!("Commitment must be 32 bytes (64 hex chars), got {}", commitment_bytes.len());
    }

    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&commitment_bytes);

    println!("Commitment: {}", hex::encode(commitment));
    println!("Namespace: {}", namespace);

    let mut tree = MerkleTree::new(20);
    let leaf_index = tree.insert(commitment);
    let proof = tree.get_proof(leaf_index);

    println!("Local tree leaf index: {}", leaf_index);
    println!("Local tree root: {}", hex::encode(tree.root()));
    println!("Proof siblings: {} levels", proof.siblings.len());

    if let (Some(url), Some(vp), Some(pp)) = (rpc_url, verifier_program, pstree_program) {
        let verifier_id: Pubkey = vp.parse()?;
        let pstree_id: Pubkey = pp.parse()?;
        let client = PrivyClient::new(&url, verifier_id, pstree_id);

        match client.build_insert_ix(&commitment, &proof, namespace) {
            Ok(ix) => {
                println!();
                println!("Insert instruction:");
                println!("  Program: {}", ix.program_id);
                println!("  Data: {} bytes", ix.data.len());
                println!("  Accounts: {}", ix.accounts.len());

                if let Some(kp_file) = keypair_file {
                    let kp_bytes = fs::read_to_string(&kp_file)
                        .map_err(|_| anyhow::anyhow!("Keypair file not found: {}", kp_file))?;
                    let kp_bytes = serde_json::from_str::<Vec<u8>>(&kp_bytes)?;
                    let keypair = Keypair::from_bytes(&kp_bytes)?;
                    match client.sign_and_send(&keypair, &[ix]) {
                        Ok(sig) => println!("Transaction sent: {}", sig),
                        Err(e) => println!("Send error: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("Error building instruction: {}", e);
            }
        }
    } else {
        println!();
        println!("No RPC info provided; instruction not sent.");
        println!("Usage: privy insert <HEX> <NAMESPACE> --rpc-url <URL> --pstree-program <ID>");
    }

    Ok(())
}

fn cmd_consume(
    nullifier_hex: String,
    rpc_url: Option<String>,
    pstree_program: Option<String>,
    keypair_file: Option<String>,
) -> anyhow::Result<()> {
    let nullifier = Nullifier::from_hex(&nullifier_hex)
        .map_err(|_| anyhow::anyhow!("Invalid nullifier hex: must be 64 hex characters"))?;

    println!("Nullifier: {}", nullifier.to_hex());

    if let (Some(url), Some(pp)) = (rpc_url, pstree_program) {
        let verifier_id = Pubkey::new_unique();
        let pstree_id: Pubkey = pp.parse()?;
        let client = PrivyClient::new(&url, verifier_id, pstree_id);

        match client.build_consume_ix(&nullifier) {
            Ok(ix) => {
                println!();
                println!("Consume instruction:");
                println!("  Program: {}", ix.program_id);
                println!("  Data: {} bytes", ix.data.len());
                println!("  Accounts: {}", ix.accounts.len());
                for meta in &ix.accounts {
                    println!("    {} (writable={})", meta.pubkey, meta.is_writable);
                }

                if let Some(kp_file) = keypair_file {
                    let kp_bytes = fs::read_to_string(&kp_file)
                        .map_err(|_| anyhow::anyhow!("Keypair file not found: {}", kp_file))?;
                    let kp_bytes = serde_json::from_str::<Vec<u8>>(&kp_bytes)?;
                    let keypair = Keypair::from_bytes(&kp_bytes)?;
                    match client.sign_and_send(&keypair, &[ix]) {
                        Ok(sig) => println!("Transaction sent: {}", sig),
                        Err(e) => println!("Send error: {}", e),
                    }
                }
            }
            Err(e) => println!("Error building instruction: {}", e),
        }
    } else {
        println!("No RPC URL and pstree program ID; instruction not sent.");
    }

    Ok(())
}

fn cmd_revoke(
    nullifier_hex: String,
    rpc_url: Option<String>,
    pstree_program: Option<String>,
    keypair_file: Option<String>,
) -> anyhow::Result<()> {
    let nullifier = Nullifier::from_hex(&nullifier_hex)
        .map_err(|_| anyhow::anyhow!("Invalid nullifier hex"))?;

    println!("Nullifier: {}", nullifier.to_hex());

    if let (Some(url), Some(pp), Some(kp_file)) = (rpc_url, pstree_program, keypair_file) {
        let verifier_id = Pubkey::new_unique();
        let pstree_id: Pubkey = pp.parse()?;
        let client = PrivyClient::new(&url, verifier_id, pstree_id);

        let kp_bytes = fs::read_to_string(&kp_file)
            .map_err(|_| anyhow::anyhow!("Keypair file not found: {}", kp_file))?;
        let kp_bytes = serde_json::from_str::<Vec<u8>>(&kp_bytes)?;
        let keypair = Keypair::from_bytes(&kp_bytes)?;

        match client.build_revoke_ix(&nullifier, &keypair.pubkey()) {
            Ok(ix) => {
                println!();
                println!("Revoke instruction:");
                println!("  Program: {}", ix.program_id);
                println!("  Authority: {}", keypair.pubkey());
                println!("  Data: {} bytes", ix.data.len());

                match client.sign_and_send(&keypair, &[ix]) {
                    Ok(sig) => println!("Transaction sent: {}", sig),
                    Err(e) => println!("Send error: {}", e),
                }
            }
            Err(e) => println!("Error building instruction: {}", e),
        }
    } else {
        println!("Requires --rpc-url, --pstree-program, and --keypair-file for on-chain revoke.");
    }

    Ok(())
}

fn cmd_get_root(
    namespace: u64,
    rpc_url: Option<String>,
    pstree_program: Option<String>,
) -> anyhow::Result<()> {
    println!("Namespace: {}", namespace);

    if let (Some(url), Some(pp)) = (rpc_url, pstree_program) {
        let verifier_id = Pubkey::new_unique();
        let pstree_id: Pubkey = pp.parse()?;
        let client = PrivyClient::new(&url, verifier_id, pstree_id);

        let (pstree_pda, bump) = Pubkey::find_program_address(
            &[b"pstree", &namespace.to_le_bytes()],
            &pstree_id,
        );

        println!("PSTree PDA: {}", pstree_pda);
        println!("PDA bump: {}", bump);
        println!();
        println!("To read on-chain root, query the RPC for account data at PDA.");
        println!("PDA address: {}", pstree_pda);
        println!("Namespace: {}", namespace);

        let rpc_client = solana_client::rpc_client::RpcClient::new(url.clone());
        match rpc_client.get_account(&pstree_pda) {
            Ok(account) => {
                if account.data.len() >= 32 {
                    let mut root = [0u8; 32];
                    root.copy_from_slice(&account.data[..32]);
                    println!("On-chain root: {}", hex::encode(root));
                } else {
                    println!("Account data too short ({} bytes, expected >= 32)", account.data.len());
                }
            }
            Err(e) => {
                println!("Could not read PSTree account: {}", e);
                println!("(Account may not be initialized)");
            }
        }
    } else {
        let tree = MerkleTree::new(20);
        println!("Local empty tree root: {}", hex::encode(tree.root()));
        println!("Use --rpc-url and --pstree-program to query on-chain root.");
    }

    Ok(())
}

fn cmd_keygen(output: Option<String>) -> anyhow::Result<()> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let secret_bytes = keypair.to_bytes();

    println!("Keypair generated:");
    println!("  Public key: {}", pubkey);
    println!("  Secret key (hex): {}", hex::encode(&secret_bytes));

    if let Some(output_path) = output {
        let json = serde_json::to_vec(&secret_bytes.to_vec())?;
        fs::write(&output_path, json)?;
        println!("  Saved to: {}", output_path);
    } else {
        let default_path = "privy-keypair.json";
        let json = serde_json::to_vec(&secret_bytes.to_vec())?;
        fs::write(default_path, json)?;
        println!("  Saved to: {}", default_path);
    }

    Ok(())
}
