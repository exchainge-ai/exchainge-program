#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;

pub use errors::ErrorCode;
pub use events::*;
pub use state::*;
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// ExchAInge Protocol - Physical AI Data Marketplace with SP1 Verification
///
/// Connects owners of robotics, drones and sensor data to AI teams that need that data
/// to train, validate, and deploy models. Features SP1 zero-knowledge proof verification
/// for hardware attestation and data authenticity.
#[program]
pub mod exchainge_protocol {
    use anchor_lang::prelude::*;
    use crate::{
        instructions,
        instructions::{AccessData, CreateListing, PurchaseLicense, VerifyData},
        state::{AccessType, LicenseType, UsageRights},
    };

    /// Create a new data listing with metadata and pricing
    pub fn create_listing(
        ctx: Context<CreateListing>,
        title: String,
        price_usdc: u64,
        license_type: LicenseType,
        hash: String,
        usage_rights: UsageRights,
        royalty_bps: u16,
        max_owners: Option<u32>,
        license_duration_days: Option<u32>,
    ) -> Result<()> {
        instructions::process_create_listing(
            ctx,
            title,
            price_usdc,
            license_type,
            hash,
            usage_rights,
            royalty_bps,
            max_owners,
            license_duration_days,
        )
    }

    /// Verify hardware data authenticity using SP1 proof
    pub fn verify_hardware_data(
        ctx: Context<VerifyData>,
        proof_bytes: Vec<u8>,
        public_values: Vec<u8>,
        commitment: [u8; 32],
    ) -> Result<()> {
        instructions::process_verify_hardware_data(ctx, proof_bytes, public_values, commitment)
    }

    /// Purchase a license for a data listing
    pub fn purchase_license(
        ctx: Context<PurchaseLicense>,
        listing_id: Pubkey,
        payment_amount: u64,
    ) -> Result<()> {
        instructions::process_purchase_license(ctx, listing_id, payment_amount)
    }

    /// Record data access for usage tracking and royalties
    pub fn access_data(
        ctx: Context<AccessData>,
        license_id: Pubkey,
        access_type: AccessType,
    ) -> Result<()> {
        instructions::process_access_data(ctx, license_id, access_type)
    }
}
