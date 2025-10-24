import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ExchaingeProgram } from "../target/types/exchainge_program";
import { expect } from "chai";
import * as crypto from "crypto";

describe("exchainge-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ExchaingeProgram as Program<ExchaingeProgram>;
  const owner = provider.wallet;

  // Helper to compute SHA-256 the same way as the smart contract
  function computeHash(fileKey: string, datasetId: number, fileSize: number): Buffer {
    const input = `${fileKey}:${datasetId}:${fileSize}`;
    return crypto.createHash('sha256').update(input).digest();
  }

  describe("register_dataset (trustless)", () => {
    it("Registers dataset with on-chain SHA-256 computation", async () => {
      const datasetId = 12345;
      const fileSize = 1024000;
      const fileKey = "test-file-key-abc123";

      // Generate new registry account
      const registryKeypair = anchor.web3.Keypair.generate();

      // Compute expected hash
      const expectedHash = computeHash(fileKey, datasetId, fileSize);

      console.log("Expected hash (computed off-chain):", expectedHash.toString('hex'));

      // Call register_dataset instruction
      const tx = await program.methods
        .registerDataset(
          new anchor.BN(datasetId),
          new anchor.BN(fileSize),
          fileKey
        )
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([registryKeypair])
        .rpc();

      console.log("Transaction signature:", tx);

      // Fetch the registry account
      const registryAccount = await program.account.dataRegistry.fetch(
        registryKeypair.publicKey
      );

      console.log("On-chain hash:", Buffer.from(registryAccount.datasetHash).toString('hex'));

      // Verify the data
      expect(registryAccount.owner.toString()).to.equal(owner.publicKey.toString());
      expect(registryAccount.internalKey).to.equal(`dataset_${datasetId}`);
      expect(registryAccount.datasetId.toNumber()).to.equal(datasetId);
      expect(registryAccount.fileSize.toNumber()).to.equal(fileSize);
      expect(registryAccount.fileKey).to.equal(fileKey);

      // Critical: Verify on-chain computed hash matches expected
      expect(Buffer.from(registryAccount.datasetHash).toString('hex')).to.equal(
        expectedHash.toString('hex')
      );

      console.log("✅ Hash verification passed - on-chain matches off-chain!");
    });

    it("Rejects empty file_key", async () => {
      const registryKeypair = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .registerDataset(
            new anchor.BN(123),
            new anchor.BN(1000),
            "" // Empty file_key
          )
          .accounts({
            registry: registryKeypair.publicKey,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([registryKeypair])
          .rpc();

        expect.fail("Should have thrown error for empty file_key");
      } catch (err) {
        expect(err.toString()).to.include("InvalidFileKey");
        console.log("✅ Correctly rejected empty file_key");
      }
    });

    it("Rejects zero file_size", async () => {
      const registryKeypair = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .registerDataset(
            new anchor.BN(123),
            new anchor.BN(0), // Zero file_size
            "valid-key"
          )
          .accounts({
            registry: registryKeypair.publicKey,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([registryKeypair])
          .rpc();

        expect.fail("Should have thrown error for zero file_size");
      } catch (err) {
        expect(err.toString()).to.include("InvalidFileSize");
        console.log("✅ Correctly rejected zero file_size");
      }
    });
  });

  describe("register_hash (cheaper)", () => {
    it("Registers with pre-computed hash", async () => {
      const internalKey = "my-custom-key-123";
      const precomputedHash = crypto.randomBytes(32);

      const registryKeypair = anchor.web3.Keypair.generate();

      const tx = await program.methods
        .registerHash(
          internalKey,
          Array.from(precomputedHash)
        )
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([registryKeypair])
        .rpc();

      console.log("Transaction signature:", tx);

      // Fetch and verify
      const registryAccount = await program.account.dataRegistry.fetch(
        registryKeypair.publicKey
      );

      expect(registryAccount.owner.toString()).to.equal(owner.publicKey.toString());
      expect(registryAccount.internalKey).to.equal(internalKey);
      expect(Buffer.from(registryAccount.datasetHash).toString('hex')).to.equal(
        precomputedHash.toString('hex')
      );
      expect(registryAccount.datasetId).to.be.null;
      expect(registryAccount.fileSize).to.be.null;
      expect(registryAccount.fileKey).to.be.null;

      console.log("✅ Pre-computed hash registration successful");
    });

    it("Rejects empty internal_key", async () => {
      const registryKeypair = anchor.web3.Keypair.generate();
      const hash = crypto.randomBytes(32);

      try {
        await program.methods
          .registerHash("", Array.from(hash))
          .accounts({
            registry: registryKeypair.publicKey,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([registryKeypair])
          .rpc();

        expect.fail("Should have thrown error for empty internal_key");
      } catch (err) {
        expect(err.toString()).to.include("InvalidInternalKey");
        console.log("✅ Correctly rejected empty internal_key");
      }
    });
  });

  describe("update_hash", () => {
    it("Updates hash for existing registry (owner only)", async () => {
      // First register
      const registryKeypair = anchor.web3.Keypair.generate();
      const originalHash = crypto.randomBytes(32);

      await program.methods
        .registerHash("update-test-key", Array.from(originalHash))
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([registryKeypair])
        .rpc();

      // Update with new hash
      const newHash = crypto.randomBytes(32);

      await program.methods
        .updateHash(Array.from(newHash))
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
        })
        .rpc();

      // Verify update
      const registryAccount = await program.account.dataRegistry.fetch(
        registryKeypair.publicKey
      );

      expect(Buffer.from(registryAccount.datasetHash).toString('hex')).to.equal(
        newHash.toString('hex')
      );

      console.log("✅ Hash update successful");
    });
  });

  describe("view_hash", () => {
    it("Views registry data without modification", async () => {
      // Register first
      const registryKeypair = anchor.web3.Keypair.generate();
      const hash = crypto.randomBytes(32);

      await program.methods
        .registerHash("view-test-key", Array.from(hash))
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([registryKeypair])
        .rpc();

      // View the data
      await program.methods
        .viewHash()
        .accounts({
          registry: registryKeypair.publicKey,
        })
        .rpc();

      console.log("✅ View instruction executed (check logs for output)");
    });
  });

  describe("close_registry", () => {
    it("Closes registry and reclaims rent (owner only)", async () => {
      // Register first
      const registryKeypair = anchor.web3.Keypair.generate();
      const hash = crypto.randomBytes(32);

      await program.methods
        .registerHash("close-test-key", Array.from(hash))
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([registryKeypair])
        .rpc();

      // Close the registry
      await program.methods
        .closeRegistry()
        .accounts({
          registry: registryKeypair.publicKey,
          owner: owner.publicKey,
        })
        .rpc();

      // Verify account is closed
      try {
        await program.account.dataRegistry.fetch(registryKeypair.publicKey);
        expect.fail("Account should be closed");
      } catch (err) {
        expect(err.toString()).to.include("Account does not exist");
        console.log("✅ Registry closed successfully");
      }
    });
  });
});
