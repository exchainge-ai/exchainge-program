# ExchAInge Protocol

Solana program for a physical AI dataset marketplace with on-chain verification, payment processing, and access control.

**Program ID:** `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`
**LOC:** ~1000 lines, ~640 excluding comments
**Status:** Compiles cleanly, audit required before mainnet

```bash
anchor build
anchor test --skip-local-validator
```

## Architecture

```
programs/exchainge-protocol/src/
├── lib.rs          (80 lines)   Program entrypoint
├── instructions.rs (423 lines)  Core logic
├── state.rs        (89 lines)   Account structures
├── errors.rs       (50 lines)   Error codes
├── events.rs       (63 lines)   Event definitions
└── test.rs         (55 lines)   Unit tests
```

## Instructions

### Platform Setup
- `initialize_platform` - One-time initialization with treasury and fee config
- `update_platform_config` - Update treasury, fees, or pause state, authority only

### Dataset Management
- `register_dataset` - Create dataset with metadata, pricing, and verification score
- `update_dataset` - Modify metadata or price, owner only
- `update_verification` - Update AI verification results, authority only

### Purchasing
- `purchase_dataset` - Buy dataset with SOL, splits 95% seller / 5% platform
- `verify_access` - Check purchase record exists, for backend download validation

## Account Structures

**PlatformConfig**: Singleton with authority, treasury, fee config, pause state, and stats.

**Dataset**: Owner, internal key, metadata URI, SHA256 hash, verification status and score, license type, price, revenue, and purchase count.

**Purchase**: Buyer, dataset, amount paid, platform fee, timestamp. One per buyer-dataset pair prevents double-purchase.

## Key Features

**Payment Flow**: Native SOL transfers only. Purchase instruction calculates fees with checked arithmetic, transfers seller revenue to owner, transfers platform fee to treasury, creates purchase record, updates stats.

**Validation**: SHA256 hash exactly 64 chars, verification score 0-100 and >50 for listing, price >= 0.0001 SOL and <= 1000 SOL, string length limits enforced.

**Security**: All arithmetic uses checked operations, Anchor constraints for access control, no custom auth logic, prevents self-purchase and double-purchase.

**Configuration**: Platform fee 5% in basis points, configurable before mainnet. Max fee capped at 20%. Emergency pause disables purchases.

## Integration Points

Frontend calls `register_dataset` after upload to R2/IPFS. Backend AI verification updates score via `update_verification`. Purchase flow creates on-chain record. Backend calls `verify_access` before generating signed download URL.

## TODOs

- Confirm platform fee percentage before mainnet, currently 5%.
- Implement SP1 zero-knowledge proof verification workflow.
- Implement hardware attestation via device signatures.
- Add comprehensive integration tests for purchase flow and error cases.

## Audit Focus

**Critical**: Payment logic in `purchase_dataset`, fee calculation with checked math, correct transfer recipients, double-purchase prevention.

**High**: Access control on update operations, input validation completeness, account size calculations.

**Medium**: Event emissions, error handling, constant values.

See AUDIT_SUMMARY.md for full audit documentation.

## Development

Tests:
```bash
cargo test --package exchainge-protocol --lib
```

Deploy to localnet:
```bash
anchor deploy
```

Update program ID after deployment in lib.rs and Anchor.toml.

**Warning**: Not audited. Do not deploy to mainnet without security audit.
