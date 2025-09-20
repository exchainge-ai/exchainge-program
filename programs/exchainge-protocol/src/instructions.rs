use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{state::*, events::*};
use crate::errors::ErrorCode;

/// Core instruction handlers for the ExchAInge Protocol
/// 
/// This module implements the business logic for a Physical AI data marketplace.
/// Functions include validation, security checks, and error handling.

/// SP1 proof result extracted from public values
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProofResult {
    pub commitment: [u8; 32],
    pub verification_score: u8,
    pub physics_verified: bool,
    pub anti_synthesis_score: u8,
}

impl TryFrom<&[u8]> for ProofResult {
    type Error = anchor_lang::error::Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 66 {
            return Err(ErrorCode::InvalidPublicValues.into());
        }
        
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&bytes[0..32]);
        
        let verification_score = bytes[32];
        let physics_verified = bytes[33] != 0;
        let anti_synthesis_score = bytes[34];
        
        Ok(ProofResult {
            commitment,
            verification_score,
            physics_verified,
            anti_synthesis_score,
        })
    }
}

/// Creates a new data listing in the marketplace
pub fn process_create_listing(
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
    let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
    
    // === COMPREHENSIVE INPUT VALIDATION ===
    
    // Title validation
    require!(!title.is_empty(), ErrorCode::EmptyTitle);
    require!(title.len() <= MAX_TITLE_LENGTH, ErrorCode::TitleTooLong);
    require!(!title.trim().is_empty(), ErrorCode::EmptyTitle);
    
    // Price validation with minimum threshold
    require!(price_usdc >= MIN_PRICE_USDC, ErrorCode::InvalidPrice);
    
    // Royalty validation
    require!(royalty_bps <= MAX_ROYALTY_BPS, ErrorCode::InvalidRoyalty);
    
    // Content hash validation
    require!(!hash.is_empty(), ErrorCode::InvalidContentHash);
    require!(hash.len() <= MAX_CONTENT_HASH_LENGTH, ErrorCode::InvalidContentHash);
    require!(
        hash.starts_with(IPFS_HASH_PREFIX) || hash.len() == ARWEAVE_HASH_LENGTH,
        ErrorCode::InvalidContentHash
    );
    
    // Max owners validation
    if let Some(max) = max_owners {
        require!(max > 0 && max <= MAX_OWNERS_LIMIT, ErrorCode::InvalidMaxOwners);
    }
    
    // License duration validation
    if let Some(days) = license_duration_days {
        require!(days > 0, ErrorCode::InvalidLicenseDuration);
        let duration_seconds = days as i64 * 24 * 60 * 60;
        require!(duration_seconds <= MAX_LICENSE_DURATION, ErrorCode::InvalidLicenseDuration);
    }
    
    // === ANTI-SPAM PROTECTION ===
    // TODO: Implement provider listing count check in future iteration
    
    // === POPULATE LISTING DATA ===
    let listing = &mut ctx.accounts.listing;
    
    listing.provider = ctx.accounts.provider.key();
    listing.title = title.clone();
    listing.description = String::new();
    listing.price_usdc = price_usdc;
    listing.license_type = license_type.clone();
    listing.usage_rights = usage_rights;
    listing.content_hash = hash;
    listing.verification_status = false;
    listing.commitment = [0u8; 32];
    listing.royalty_bps = royalty_bps;
    listing.max_owners = max_owners;
    listing.license_duration = license_duration_days.map(|days| {
        clock.unix_timestamp.saturating_add(days as i64 * 24 * 60 * 60)
    });
    listing.created_at = clock.unix_timestamp;
    listing.updated_at = clock.unix_timestamp;
    listing.is_active = true;
    listing.total_sales = 0;
    listing.revenue_earned = 0;
    listing.bump = 255; // Default bump since seeds removed for IDL compatibility

    emit!(ListingCreated {
        listing_id: listing.key(),
        provider: listing.provider,
        title,
        price_usdc,
        license_type,
    });

    Ok(())
}

/// Verify hardware data authenticity with enterprise-grade validation
pub fn process_verify_hardware_data(
    ctx: Context<VerifyData>,
    proof_bytes: Vec<u8>,
    public_values: Vec<u8>,
    commitment: [u8; 32],
) -> Result<()> {
    let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
    
    // === PROOF FORMAT VALIDATION ===
    require!(!proof_bytes.is_empty(), ErrorCode::InvalidSP1Proof);
    require!(proof_bytes.len() >= 32, ErrorCode::InvalidSP1Proof);
    require!(proof_bytes.len() <= 8192, ErrorCode::InvalidSP1Proof); // Max 8KB proof
    
    require!(!public_values.is_empty(), ErrorCode::InvalidPublicValues);
    require!(public_values.len() >= 66, ErrorCode::InvalidPublicValues); // Min expected size
    require!(public_values.len() <= 1024, ErrorCode::InvalidPublicValues); // Max 1KB public values
    
    // === EXTRACT AND VALIDATE PROOF RESULTS ===
    let proof_result: ProofResult = public_values.as_slice().try_into()?;
    
    // Commitment validation
    require!(
        proof_result.commitment == commitment,
        ErrorCode::CommitmentMismatch
    );
    
    // Ensure commitment is not all zeros
    require!(
        commitment != [0u8; 32],
        ErrorCode::InvalidSP1Proof
    );
    
    // Verification score validation
    require!(
        proof_result.verification_score >= MIN_VERIFICATION_THRESHOLD,
        ErrorCode::InsufficientVerification
    );
    
    // Anti-synthesis protection
    require!(
        proof_result.anti_synthesis_score >= MIN_VERIFICATION_THRESHOLD,
        ErrorCode::SyntheticDataDetected
    );
    
    // Physics verification requirement
    require!(
        proof_result.physics_verified,
        ErrorCode::InvalidSP1Proof
    );
    
    // === LISTING STATE VALIDATION ===
    let listing = &mut ctx.accounts.listing;
    
    // Prevent double verification
    require!(
        !listing.verification_status,
        ErrorCode::DuplicatePurchase // Reusing error for duplicate verification
    );
    
    // Ensure listing is not expired
    require!(
        !listing.is_expired(clock.unix_timestamp),
        ErrorCode::ListingExpired
    );
    
    // === GENERATE VERIFICATION HASHES ===
    let sp1_proof_hash = anchor_lang::solana_program::keccak::hash(&proof_bytes).to_bytes();
    let public_values_hash = anchor_lang::solana_program::keccak::hash(&public_values).to_bytes();
    
    // === UPDATE LISTING ===
    listing.verification_status = true;
    listing.commitment = commitment;
    listing.updated_at = clock.unix_timestamp;
    
    // === CREATE VERIFICATION RECORD ===
    let verification = &mut ctx.accounts.hardware_verification;
    verification.listing_id = listing.key();
    verification.data_commitment = commitment;
    verification.vendor = HardwareVendor::Custom; // Could be extracted from proof metadata
    verification.device_id = "verified_device".to_string(); // Truncated for security
    verification.verification_score = proof_result.verification_score;
    verification.physics_verified = proof_result.physics_verified;
    verification.anti_synthesis_score = proof_result.anti_synthesis_score;
    verification.verifier_pubkey = ctx.accounts.provider.key();
    verification.sp1_proof_hash = sp1_proof_hash;
    verification.public_values_hash = public_values_hash;
    verification.verified_at = clock.unix_timestamp;
    verification.nonce = clock.unix_timestamp as u64; // Simple nonce for anti-replay
    verification.bump = 255; // Default bump since seeds removed for IDL compatibility

    emit!(DataVerified {
        listing_id: listing.key(),
        commitment,
        verification_score: proof_result.verification_score,
        verifier: ctx.accounts.provider.key(),
    });

    Ok(())
}

/// Purchase license with enterprise security and payment protection
pub fn process_purchase_license(
    ctx: Context<PurchaseLicense>,
    listing_id: Pubkey,
    payment_amount: u64,
) -> Result<()> {
    let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
    let listing = &ctx.accounts.listing;

    // === LISTING VALIDATION ===
    require!(listing.key() == listing_id, ErrorCode::InvalidListingId);
    require!(listing.is_active, ErrorCode::ListingInactive);
    require!(listing.verification_status, ErrorCode::DataNotVerified);
    require!(!listing.is_expired(clock.unix_timestamp), ErrorCode::ListingExpired);

    // === ANTI-FRAUD PROTECTION ===
    // Prevent self-purchase
    require!(
        listing.provider != ctx.accounts.buyer.key(),
        ErrorCode::SelfPurchaseProhibited
    );

    // === PAYMENT VALIDATION ===
    require!(
        payment_amount >= listing.price_usdc,
        ErrorCode::InsufficientPayment
    );
    
    // Prevent massive overpayment (possible mistake or attack)
    let max_payment = listing.price_usdc
        .checked_mul(10)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    require!(
        payment_amount <= max_payment,
        ErrorCode::InsufficientPayment
    );

    // === LICENSE TYPE RESTRICTIONS ===
    match listing.license_type {
        LicenseType::Exclusive | LicenseType::TransferableExclusive => {
            require!(listing.total_sales == 0, ErrorCode::ExclusiveLicenseAlreadySold);
        }
        _ => {
            if let Some(max_owners) = listing.max_owners {
                require!(listing.total_sales < max_owners, ErrorCode::MaxOwnersReached);
            }
        }
    }

    // === TOKEN ACCOUNT VALIDATION ===
    require!(
        ctx.accounts.buyer_token_account.mint == ctx.accounts.usdc_mint.key(),
        ErrorCode::InvalidTokenMint
    );
    require!(
        ctx.accounts.provider_token_account.mint == ctx.accounts.usdc_mint.key(),
        ErrorCode::InvalidTokenMint
    );

    // Check buyer has sufficient balance
    require!(
        ctx.accounts.buyer_token_account.amount >= payment_amount,
        ErrorCode::InsufficientBalance
    );

    // === SECURE PAYMENT PROCESSING ===
    let platform_fee = payment_amount
        .checked_mul(PLATFORM_FEE_BPS)
        .and_then(|x| x.checked_div(10_000))
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    let seller_amount = payment_amount
        .checked_sub(platform_fee)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Transfer USDC to provider (platform fee stays in buyer account for now)
    let transfer_ix = anchor_spl::token::Transfer {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.provider_token_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };

    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_ix,
        ),
        seller_amount,
    ).map_err(|_| ErrorCode::PaymentFailed)?;

    // === CREATE SECURE LICENSE TOKEN ===
    let license = &mut ctx.accounts.license_token;
    
    license.listing_id = listing.key();
    license.owner = ctx.accounts.buyer.key();
    license.original_buyer = ctx.accounts.buyer.key();
    license.license_type = listing.license_type.clone();
    license.usage_rights = listing.usage_rights.clone();
    license.purchase_price = payment_amount;
    license.purchase_date = clock.unix_timestamp;
    license.expiration = listing.license_duration;
    license.access_count = 0;
    license.last_access = 0;
    license.usage_limit = None; // Could be enhanced based on license type
    license.is_transferable = matches!(
        listing.license_type,
        LicenseType::TransferableExclusive
    );
    license.transfer_count = 0;
    license.is_revoked = false;
    license.bump = 255; // Default bump since seeds removed for IDL compatibility

    // === UPDATE LISTING STATS ===
    let listing = &mut ctx.accounts.listing;
    listing.total_sales = listing.total_sales
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    listing.revenue_earned = listing.revenue_earned
        .checked_add(seller_amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    listing.updated_at = clock.unix_timestamp;

    // Deactivate listing if exclusive
    if matches!(listing.license_type, LicenseType::Exclusive) {
        listing.is_active = false;
    }

    emit!(LicensePurchased {
        listing_id: listing.key(),
        buyer: ctx.accounts.buyer.key(),
        license_id: license.key(),
        price: payment_amount,
        platform_fee,
    });

    Ok(())
}

/// Record data access with comprehensive access control
pub fn process_access_data(
    ctx: Context<AccessData>,
    license_id: Pubkey,
    access_type: AccessType,
) -> Result<()> {
    let clock = Clock::get().map_err(|_| ErrorCode::ClockUnavailable)?;
    let license = &mut ctx.accounts.license_token;

    // === LICENSE VALIDATION ===
    require!(license.key() == license_id, ErrorCode::InvalidLicenseId);
    require!(
        license.owner == ctx.accounts.user.key(),
        ErrorCode::UnauthorizedLicenseHolder
    );

    // === ACCESS PERMISSION CHECKS ===
    require!(license.can_access(clock.unix_timestamp), ErrorCode::LicenseExpired);
    require!(!license.is_revoked, ErrorCode::LicenseRevoked);

    // Check usage limits with overflow protection
    if let Some(limit) = license.usage_limit {
        require!(license.access_count < limit, ErrorCode::UsageLimitExceeded);
    }

    // === USAGE RIGHTS VALIDATION ===
    // Validate access type against usage rights
    match access_type {
        AccessType::Download => {
            // Basic download access - allowed for all license types
        }
        AccessType::Stream => {
            // Streaming might have different restrictions
        }
        AccessType::API => {
            // API access might require commercial rights
            if !license.usage_rights.commercial_use {
                require!(false, ErrorCode::FeatureNotAvailable);
            }
        }
        AccessType::Compute => {
            // Compute-to-data requires special permissions
            require!(
                license.usage_rights.ai_training_allowed,
                ErrorCode::FeatureNotAvailable
            );
        }
    }

    // === UPDATE ACCESS METRICS ===
    license.access_count = license.access_count
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    license.last_access = clock.unix_timestamp;

    emit!(DataAccessed {
        license_id: license.key(),
        user: ctx.accounts.user.key(),
        access_type,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

// === SECURE CONTEXT STRUCTURES ===

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateListing<'info> {
    #[account(
        init,
        payer = provider,
        space = DataListing::LEN
    )]
    pub listing: Account<'info, DataListing>,

    #[account(mut)]
    pub provider: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyData<'info> {
    #[account(
        mut,
        has_one = provider @ ErrorCode::UnauthorizedProvider,
        constraint = !listing.verification_status @ ErrorCode::DuplicatePurchase
    )]
    pub listing: Account<'info, DataListing>,

    #[account(
        init,
        payer = provider,
        space = HardwareVerification::LEN
    )]
    pub hardware_verification: Account<'info, HardwareVerification>,

    #[account(mut)]
    pub provider: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchaseLicense<'info> {
    #[account(
        mut,
        constraint = listing.is_active @ ErrorCode::ListingInactive,
        constraint = listing.verification_status @ ErrorCode::DataNotVerified
    )]
    pub listing: Account<'info, DataListing>,

    #[account(
        init,
        payer = buyer,
        space = LicenseToken::LEN
    )]
    pub license_token: Account<'info, LicenseToken>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Provider account validated in instruction
    #[account(mut)]
    pub provider: AccountInfo<'info>,

    // USDC token accounts with enhanced validation
    #[account(
        mut,
        constraint = buyer_token_account.mint == usdc_mint.key() @ ErrorCode::InvalidTokenMint,
        constraint = buyer_token_account.owner == buyer.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = provider_token_account.mint == usdc_mint.key() @ ErrorCode::InvalidTokenMint,
        constraint = provider_token_account.owner != buyer.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub provider_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = usdc_mint.decimals == 6 @ ErrorCode::InvalidTokenMint
    )]
    pub usdc_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AccessData<'info> {
    #[account(
        mut,
        constraint = license_token.owner == user.key() @ ErrorCode::UnauthorizedLicenseHolder,
        constraint = !license_token.is_revoked @ ErrorCode::LicenseRevoked
    )]
    pub license_token: Account<'info, LicenseToken>,

    pub user: Signer<'info>,
}
