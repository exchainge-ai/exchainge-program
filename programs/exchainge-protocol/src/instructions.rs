use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::{
    errors::ErrorCode,
    events::*,
    state::*,
};

// Initialize platform configuration. Called once on deployment.
pub fn initialize_platform(
    ctx: Context<InitializePlatform>,
    treasury: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.authority = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.fee_bps = PLATFORM_FEE_BPS;
    config.paused = false;
    config.total_platform_revenue = 0;
    config.total_datasets = 0;
    config.total_purchases = 0;
    config.bump = ctx.bumps.config;

    emit!(PlatformInitialized {
        authority: config.authority,
        treasury: config.treasury,
        fee_bps: config.fee_bps,
    });

    Ok(())
}

// Update platform settings. Authority only.
pub fn update_platform_config(
    ctx: Context<UpdatePlatformConfig>,
    new_treasury: Option<Pubkey>,
    new_fee_bps: Option<u64>,
    paused: Option<bool>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    if let Some(treasury) = new_treasury {
        config.treasury = treasury;
    }

    if let Some(fee_bps) = new_fee_bps {
        require!(fee_bps <= 2000, ErrorCode::PriceTooHigh); // TODO: Confirm max 20% fee.
        config.fee_bps = fee_bps;
    }

    if let Some(p) = paused {
        config.paused = p;
    }

    emit!(PlatformConfigUpdated {
        authority: config.authority,
        treasury: config.treasury,
        fee_bps: config.fee_bps,
        paused: config.paused,
    });

    Ok(())
}

// Register new dataset with metadata and pricing.
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
    let dataset = &mut ctx.accounts.dataset;
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;

    require!(!config.paused, ErrorCode::PlatformPaused);

    let internal_key = internal_key.trim().to_string();
    require!(
        !internal_key.is_empty() && internal_key.len() <= MAX_INTERNAL_KEY_LENGTH,
        ErrorCode::InvalidInternalKey
    );

    let metadata_uri = metadata_uri.trim().to_string();
    require!(
        !metadata_uri.is_empty() && metadata_uri.len() <= MAX_URI_LENGTH,
        ErrorCode::InvalidUri
    );

    require!(data_hash.len() == 64, ErrorCode::InvalidContentHash);
    require!(price_lamports >= MIN_PRICE_LAMPORTS, ErrorCode::PriceTooLow);
    require!(price_lamports <= 1_000_000_000_000, ErrorCode::PriceTooHigh);
    require!(verification_score <= 100, ErrorCode::InvalidScore);
    require!(verification_score > 50, ErrorCode::LowScore);

    dataset.owner = ctx.accounts.owner.key();
    dataset.internal_key = internal_key.clone();
    dataset.metadata_uri = metadata_uri;
    dataset.data_hash = data_hash.clone();
    dataset.verified = true;
    dataset.verifier_type = verifier_type;
    dataset.verification_score = verification_score;
    dataset.license_type = license_type;
    dataset.price_lamports = price_lamports;
    dataset.seller_revenue = 0;
    dataset.purchase_count = 0;
    dataset.created_at = clock.unix_timestamp;
    dataset.updated_at = clock.unix_timestamp;
    dataset.bump = 0;

    config.total_datasets = config
        .total_datasets
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    emit!(DatasetRegistered {
        dataset: dataset.key(),
        owner: dataset.owner,
        internal_key,
        data_hash,
        price_lamports,
        verifier_type,
        verification_score,
        license_type,
    });

    Ok(())
}

// Update dataset metadata or price. Owner only.
pub fn update_dataset(
    ctx: Context<UpdateDataset>,
    new_metadata_uri: Option<String>,
    new_price_lamports: Option<u64>,
) -> Result<()> {
    let dataset = &mut ctx.accounts.dataset;
    let clock = Clock::get()?;

    if let Some(uri) = new_metadata_uri {
        require!(
            !uri.trim().is_empty() && uri.len() <= MAX_URI_LENGTH,
            ErrorCode::InvalidUri
        );
        dataset.metadata_uri = uri.trim().to_string();
    }

    if let Some(price) = new_price_lamports {
        require!(price >= MIN_PRICE_LAMPORTS, ErrorCode::PriceTooLow);
        require!(price <= 1_000_000_000_000, ErrorCode::PriceTooHigh);
        dataset.price_lamports = price;
    }

    dataset.updated_at = clock.unix_timestamp;

    emit!(DatasetUpdated {
        dataset: dataset.key(),
        metadata_uri: dataset.metadata_uri.clone(),
        price_lamports: dataset.price_lamports,
    });

    Ok(())
}

// Update verification status after off-chain AI verification. Authority only.
// TODO: Implement full AI and SP1 verification workflows.
pub fn update_verification(
    ctx: Context<UpdateVerification>,
    verifier_type: VerifierType,
    verification_score: u8,
) -> Result<()> {
    let dataset = &mut ctx.accounts.dataset;
    let clock = Clock::get()?;

    require!(verification_score <= 100, ErrorCode::InvalidScore);

    dataset.verifier_type = verifier_type;
    dataset.verification_score = verification_score;
    dataset.verified = verification_score > 50;
    dataset.updated_at = clock.unix_timestamp;

    emit!(DatasetVerified {
        dataset: dataset.key(),
        verifier_type,
        verification_score,
    });

    Ok(())
}

// Purchase dataset with SOL payment. Splits payment 95% seller, 5% platform(could change tbd).
// Uses native system program transfers only.
pub fn purchase_dataset(ctx: Context<PurchaseDataset>) -> Result<()> {
    let config = &ctx.accounts.config;
    let dataset = &mut ctx.accounts.dataset;
    let purchase = &mut ctx.accounts.purchase;
    let clock = Clock::get()?;

    require!(!config.paused, ErrorCode::PlatformPaused);
    require!(dataset.verified, ErrorCode::DatasetNotVerified);
    require!(
        ctx.accounts.buyer.key() != dataset.owner,
        ErrorCode::SelfPurchaseNotAllowed
    );

    // Calculate fees with checked arithmetic.
    let total_price = dataset.price_lamports;
    let platform_fee = total_price
        .checked_mul(config.fee_bps)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::DivisionByZero)?;
    let seller_revenue = total_price
        .checked_sub(platform_fee)
        .ok_or(ErrorCode::MathUnderflow)?;

    // Transfer seller revenue.
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        seller_revenue,
    )?;

    // Transfer platform fee to treasury.
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
            },
        ),
        platform_fee,
    )?;

    dataset.seller_revenue = dataset
        .seller_revenue
        .checked_add(seller_revenue)
        .ok_or(ErrorCode::MathOverflow)?;
    dataset.purchase_count = dataset
        .purchase_count
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    purchase.buyer = ctx.accounts.buyer.key();
    purchase.dataset = dataset.key();
    purchase.amount_paid = total_price;
    purchase.platform_fee = platform_fee;
    purchase.purchased_at = clock.unix_timestamp;
    purchase.bump = 0;

    let config = &mut ctx.accounts.config;
    config.total_platform_revenue = config
        .total_platform_revenue
        .checked_add(platform_fee)
        .ok_or(ErrorCode::MathOverflow)?;
    config.total_purchases = config
        .total_purchases
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    emit!(DatasetPurchased {
        purchase: purchase.key(),
        dataset: dataset.key(),
        buyer: ctx.accounts.buyer.key(),
        seller: dataset.owner,
        total_amount: total_price,
        platform_fee,
        seller_revenue,
    });

    emit!(AccessGranted {
        buyer: ctx.accounts.buyer.key(),
        dataset: dataset.key(),
        expires_at: 0, // Lifetime access
    });

    Ok(())
}

// Verify purchase exists. Backend calls this before generating download URL.
pub fn verify_access(ctx: Context<VerifyAccess>) -> Result<()> {
    let purchase = &ctx.accounts.purchase;

    msg!(
        "Access verified: buyer={}, dataset={}, purchased_at={}",
        purchase.buyer,
        purchase.dataset,
        purchase.purchased_at
    );

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PlatformConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlatformConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub config: Account<'info, PlatformConfig>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(internal_key: String)]
pub struct RegisterDataset<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Dataset::INIT_SPACE
    )]
    pub dataset: Account<'info, Dataset>,

    #[account(mut)]
    pub config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateDataset<'info> {
    #[account(
        mut,
        has_one = owner @ ErrorCode::NotDatasetOwner
    )]
    pub dataset: Account<'info, Dataset>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateVerification<'info> {
    #[account(mut)]
    pub dataset: Account<'info, Dataset>,

    #[account(
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub config: Account<'info, PlatformConfig>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PurchaseDataset<'info> {
    #[account(
        init,
        payer = buyer,
        space = 8 + Purchase::INIT_SPACE
    )]
    pub purchase: Account<'info, Purchase>,

    #[account(mut)]
    pub dataset: Account<'info, Dataset>,

    #[account(mut)]
    pub config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Dataset owner validated via constraint.
    #[account(
        mut,
        constraint = seller.key() == dataset.owner @ ErrorCode::NotDatasetOwner
    )]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Platform treasury validated via constraint.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::Unauthorized
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyAccess<'info> {
    pub purchase: Account<'info, Purchase>,

    #[account(
        constraint = dataset.key() == purchase.dataset @ ErrorCode::NoPurchaseRecord
    )]
    pub dataset: Account<'info, Dataset>,
}
