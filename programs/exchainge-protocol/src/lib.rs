use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod exchainge_protocol {
    use super::*;

    pub fn initialize_platform(
        ctx: Context<InitializePlatform>,
        treasury: Pubkey,
    ) -> Result<()> {
        instructions::initialize_platform(ctx, treasury)
    }

    pub fn update_platform_config(
        ctx: Context<UpdatePlatformConfig>,
        new_treasury: Option<Pubkey>,
        new_fee_bps: Option<u64>,
        paused: Option<bool>,
    ) -> Result<()> {
        instructions::update_platform_config(ctx, new_treasury, new_fee_bps, paused)
    }

    pub fn register_dataset(
        ctx: Context<RegisterDataset>,
        internal_key: String,
        metadata_uri: String,
        data_hash: String,
        price_lamports: u64,
        license_type: LicenseType,
        verifier_type: VerifierType,
        verification_score: u8,
    ) -> Result<()> {
        instructions::register_dataset(
            ctx,
            internal_key,
            metadata_uri,
            data_hash,
            price_lamports,
            license_type,
            verifier_type,
            verification_score,
        )
    }

    pub fn update_dataset(
        ctx: Context<UpdateDataset>,
        new_metadata_uri: Option<String>,
        new_price_lamports: Option<u64>,
    ) -> Result<()> {
        instructions::update_dataset(ctx, new_metadata_uri, new_price_lamports)
    }

    pub fn update_verification(
        ctx: Context<UpdateVerification>,
        verifier_type: VerifierType,
        verification_score: u8,
    ) -> Result<()> {
        instructions::update_verification(ctx, verifier_type, verification_score)
    }

    pub fn purchase_dataset(ctx: Context<PurchaseDataset>) -> Result<()> {
        instructions::purchase_dataset(ctx)
    }

    pub fn verify_access(ctx: Context<VerifyAccess>) -> Result<()> {
        instructions::verify_access(ctx)
    }
}
