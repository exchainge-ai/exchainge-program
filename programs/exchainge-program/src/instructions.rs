use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};
use crate::state::*;
use crate::errors::ErrorCode;
use crate::events::*;

// ========================================
// Instruction 1: register_dataset (Trustless)
// ========================================

/// Register dataset with on-chain SHA-256 hash computation
/// More secure and trustless - hash is computed on-chain from inputs
pub fn process_register_dataset(
    ctx: Context<RegisterDataset>,
    dataset_id: u64,
    file_size: u64,
    file_key: String,
) -> Result<()> {
    // Validate inputs
    require!(!file_key.is_empty() && file_key.len() <= 100, ErrorCode::InvalidFileKey);
    require!(file_size > 0, ErrorCode::InvalidFileSize);

    // Compute SHA-256 hash on-chain: sha256(file_key:dataset_id:file_size)
    let hash_input = format!("{}:{}:{}", file_key, dataset_id, file_size);
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let derived_hash: [u8; 32] = hasher.finalize().into();

    // Store in registry account
    let registry = &mut ctx.accounts.registry;
    let clock = Clock::get()?;

    registry.owner = ctx.accounts.owner.key();
    registry.internal_key = format!("dataset_{}", dataset_id);
    registry.dataset_hash = derived_hash;
    registry.dataset_id = Some(dataset_id);
    registry.file_size = Some(file_size);
    registry.file_key = Some(file_key.clone());
    registry.created_at = clock.unix_timestamp;
    registry.bump = 0;

    // Emit event
    emit!(DatasetRegistered {
        dataset_id,
        file_size,
        file_key,
        derived_hash,
        owner: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Dataset registered (trustless): ID={}, Hash={:?}",
        dataset_id,
        derived_hash
    );

    Ok(())
}

// ========================================
// Instruction 2: register_hash (Cheaper)
// ========================================

/// Register with pre-computed hash (cheaper, requires trust in client)
pub fn process_register_hash(
    ctx: Context<RegisterHash>,
    internal_key: String,
    dataset_hash: [u8; 32],
) -> Result<()> {
    // Validate inputs
    require!(!internal_key.is_empty() && internal_key.len() <= 64, ErrorCode::InvalidInternalKey);

    // Store in registry account
    let registry = &mut ctx.accounts.registry;
    let clock = Clock::get()?;

    registry.owner = ctx.accounts.owner.key();
    registry.internal_key = internal_key.clone();
    registry.dataset_hash = dataset_hash;
    registry.dataset_id = None;
    registry.file_size = None;
    registry.file_key = None;
    registry.created_at = clock.unix_timestamp;
    registry.bump = 0;

    // Emit event
    emit!(HashRegistered {
        internal_key: internal_key.clone(),
        dataset_hash,
        owner: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Hash registered (pre-computed): Key={}, Hash={:?}",
        internal_key,
        dataset_hash
    );

    Ok(())
}

// ========================================
// Instruction 3: update_hash
// ========================================

/// Update the hash for an existing registry entry (owner only)
pub fn process_update_hash(
    ctx: Context<UpdateHash>,
    new_dataset_hash: [u8; 32],
) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    let clock = Clock::get()?;

    registry.dataset_hash = new_dataset_hash;

    // Emit event
    emit!(RegistryUpdated {
        registry_address: ctx.accounts.registry.key(),
        new_hash: new_dataset_hash,
        owner: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Registry updated: {:?}", new_dataset_hash);

    Ok(())
}

// ========================================
// Instruction 4: view_hash (Read-only)
// ========================================

/// View/fetch registry data by registry account address
/// This is a read instruction - just fetches account data
/// In practice, clients can also directly fetch the account using Connection.getAccountInfo()
pub fn process_view_hash(ctx: Context<ViewHash>) -> Result<()> {
    let registry = &ctx.accounts.registry;

    msg!("Registry data:");
    msg!("  Owner: {}", registry.owner);
    msg!("  Internal Key: {}", registry.internal_key);
    msg!("  Dataset Hash: {:?}", registry.dataset_hash);
    msg!("  Dataset ID: {:?}", registry.dataset_id);
    msg!("  File Size: {:?}", registry.file_size);
    msg!("  File Key: {:?}", registry.file_key);
    msg!("  Created At: {}", registry.created_at);

    Ok(())
}

// ========================================
// Instruction 5: close_registry
// ========================================

/// Close a registry and reclaim rent (owner only)
pub fn process_close_registry(ctx: Context<CloseRegistry>) -> Result<()> {
    let clock = Clock::get()?;

    // Emit event
    emit!(RegistryClosed {
        registry_address: ctx.accounts.registry.key(),
        owner: ctx.accounts.owner.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Registry closed, rent reclaimed");
    Ok(())
}

// ========================================
// Account Validation Structs
// ========================================

#[derive(Accounts)]
pub struct RegisterDataset<'info> {
    #[account(
        init,
        payer = owner,
        space = DataRegistry::LEN
    )]
    pub registry: Account<'info, DataRegistry>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterHash<'info> {
    #[account(
        init,
        payer = owner,
        space = DataRegistry::LEN
    )]
    pub registry: Account<'info, DataRegistry>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateHash<'info> {
    #[account(
        mut,
        constraint = registry.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, DataRegistry>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ViewHash<'info> {
    pub registry: Account<'info, DataRegistry>,
}

#[derive(Accounts)]
pub struct CloseRegistry<'info> {
    #[account(
        mut,
        close = owner,
        constraint = registry.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, DataRegistry>,

    #[account(mut)]
    pub owner: Signer<'info>,
}
