# ExchAInge Protocol

Solana program for Physical AI data marketplace with SP1 zero knowledge proof verification.

Robotics and drone operators sell verified data to AI teams. Frontend generates SP1 proofs, program validates and handles USDC payments.

## Setup

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Install Anchor
npm install -g @coral-xyz/anchor-cli@0.31.1

# Build and deploy
npm run build
npm run deploy-localnet
```

## Repository Structure

```
programs/exchainge-protocol/src/
├── lib.rs              // Program entry point, instruction routing
├── state.rs            // Account structures (DataListing, HardwareVerification, LicenseToken)  
├── instructions.rs     // Business logic for create/verify/purchase/access
├── errors.rs           // Error codes
└── events.rs           // Event definitions for indexing

Anchor.toml            // Program configuration
package.json           // Build scripts
```

## Core Concepts

**DataListing** Marketplace entry with price, metadata, verification status
**HardwareVerification** Immutable SP1 proof validation record  
**LicenseToken** NFT style license with usage rights and access control

## Frontend Integration

### create_listing
```typescript
await program.methods
  .createListing(
    title,           // string, max 100 chars
    priceUsdc,       // u64, in microunits (1000 = $0.001)  
    licenseType,     // enum: ViewOnly | Exclusive | etc
    contentHash,     // string, IPFS/Arweave hash
    usageRights,     // struct with permissions
    royaltyBps,      // u16, max 5000 (50%)
    maxOwners,       // Option<u32>
    durationDays     // Option<u32>
  )
  .accounts({
    listing: listingPda,
    provider: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### verify_hardware_data  
```typescript
await program.methods
  .verifyHardwareData(
    proofBytes,      // Vec<u8>, raw SP1 proof
    publicValues,    // Vec<u8>, proof inputs
    commitment       // [u8; 32], data commitment
  )
  .accounts({
    listing: listingPda,
    hardwareVerification: verificationPda,
    provider: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### purchase_license
```typescript
await program.methods
  .purchaseLicense(
    listingId,       // Pubkey
    paymentAmount    // u64, USDC amount
  )
  .accounts({
    listing: listingPda,
    licenseToken: licensePda,
    buyer: wallet.publicKey,
    provider: providerPubkey,
    buyerTokenAccount: buyerUsdcAccount,
    providerTokenAccount: providerUsdcAccount,
    usdcMint: USDC_MINT,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### access_data
```typescript
await program.methods
  .accessData(
    licenseId,       // Pubkey
    accessType       // enum: Download | Stream | API | Compute
  )
  .accounts({
    licenseToken: licensePda,
    user: wallet.publicKey,
  })
  .rpc();
```

## PDA Generation

```typescript
// Listing PDA
const [listingPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("listing"), provider.toBuffer(), Buffer.from(title)],
  programId
);

// License PDA  
const [licensePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("license"), listingPda.toBuffer(), buyer.toBuffer()],
  programId
);

// Verification PDA
const [verificationPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("verification"), listingPda.toBuffer()],
  programId
);
```

## Development

### Prerequisites
- Solana CLI v1.17+
- Anchor Framework v0.31.1  
- Node.js v18+
- Rust with cargo build sbf

### Local Testing
```bash
# Terminal 1: Start validator
solana-test-validator --reset

# Terminal 2: Build and deploy
npm run build
npm run deploy-localnet

# Verify deployment
solana program show [PROGRAM_ID]
```

### Account Sizes
- DataListing: 854 bytes (0.006 SOL rent)
- HardwareVerification: 257 bytes (0.002 SOL rent)  
- LicenseToken: 149 bytes (0.001 SOL rent)

### Key Configuration (src/state.rs)
- PLATFORM_FEE_BPS: 300 (3% fee)
- MIN_PRICE_USDC: 1000 (minimum $0.001)
- MIN_VERIFICATION_THRESHOLD: 60 (verification score)
- MAX_ROYALTY_BPS: 5000 (max 50% royalty)

### Common Errors
- InvalidPrice: Below minimum threshold
- InsufficientPayment: Payment less than listing price
- ExclusiveLicenseAlreadySold: Exclusive license unavailable
- InvalidSP1Proof: Proof validation failed
- UnauthorizedAccess: User lacks permissions

### Events
- ListingCreated: New marketplace entry
- DataVerified: SP1 verification success
- LicensePurchased: License sale completed
- DataAccessed: Data usage recorded

### Deployment
- Localnet: `npm run deploy-localnet`
- Devnet: `npm run deploy-devnet`
- Mainnet: `npm run deploy-mainnet`

Program ID changes per deployment. Update frontend accordingly.

### Security Features
- Checked arithmetic for financial calculations
- Input validation on string lengths and numeric ranges
- SP1 proof verification via hash validation
- Platform fee automatically deducted from payments
- License transfers limited to prevent abuse