/**
 * Solana Anchor Test Configuration
 * Run with: anchor test --skip-build
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PrivyVerifier } from "../target/types/privy_verifier";
import { PrivyPstree } from "../target/types/privy_pstree";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { BN } from "bn.js";
import { assert } from "chai";

// Workaround: Solana test requires describe/it from Mocha
declare const describe: any;
declare const it: any;
declare const before: any;

describe("Privy SVM — Anchor Integration Tests", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  describe("privy-pstree: Private State Tree", () => {
    const program = anchor.workspace.PrivyPstree as Program<PrivyPstree>;
    const namespace = new BN(1);
    let treePda: PublicKey;

    before(async () => {
      [treePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("pstree"), namespace.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
    });

    it("initialize_tree creates a new tree", async () => {
      await program.methods
        .initializeTree(namespace)
        .accounts({ signer: provider.wallet.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      
      const tree = await program.account.pstreeAccount.fetch(treePda);
      assert.equal(tree.leafCount.toNumber(), 0);
      console.log("  ✓ Tree initialized, root:", Buffer.from(tree.root).toString("hex").slice(0, 16));
    });

    it("insert_commitment adds a leaf", async () => {
      const leafIndex = new BN(0);
      const commit = Array.from(Buffer.alloc(32, 0x42));
      const nullifier = Array.from(Buffer.alloc(32, 0x99));
      const expiry = new BN(0);
      
      // Empty tree proof of depth 256
      // For a real test, this would compute Poseidon hashes matching the on-chain program
      // Here we provide 256 zero-filled sibling entries (empty tree assumption)
      const proof: number[][] = [];
      let node = new Uint8Array(32);
      for (let i = 0; i < 256; i++) {
        proof.push(Array.from(node));
        // Simplified: node = Poseidon(node, node)
        node = new Uint8Array(node); // placeholder
      }
      
      await program.methods
        .insertCommitment(leafIndex, commit, nullifier, expiry, proof)
        .accounts({ signer: provider.wallet.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      
      const tree = await program.account.pstreeAccount.fetch(treePda);
      assert.equal(tree.leafCount.toNumber(), 1);
      console.log("  ✓ Leaf inserted, count:", tree.leafCount.toNumber());
    });
  });

  describe("privy-verifier: ZK Proof Verifier", () => {
    const program = anchor.workspace.PrivyVerifier as Program<PrivyVerifier>;
    let verifierPda: PublicKey;

    before(async () => {
      [verifierPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("privy-verifier")],
        program.programId
      );
    });

    it("initialize creates the verifier state", async () => {
      await program.methods
        .initialize()
        .accounts({ admin: provider.wallet.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      
      const state = await program.account.verifierState.fetch(verifierPda);
      assert.equal(state.totalVerifications.toNumber(), 0);
      console.log("  ✓ Verifier initialized, admin:", state.admin.toBase58());
    });
  });
});
