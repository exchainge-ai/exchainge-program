#![cfg(test)]

use super::build_hardware_signature_message;
use crate::types::HARDWARE_SIGNATURE_MAX_AGE;
use anchor_lang::prelude::Pubkey;
use ed25519_dalek::{Keypair, PublicKey as DalekPublicKey, SecretKey, Signer, Verifier};

fn fixture_keypair() -> Keypair {
    let secret_bytes = [7u8; 32];
    let secret = SecretKey::from_bytes(&secret_bytes).expect("static secret key");
    let public = DalekPublicKey::from(&secret);
    Keypair { secret, public }
}

#[test]
fn hardware_signature_message_round_trip() {
    let listing = Pubkey::new_unique();
    let data_hash = "sensor_hash".to_string();
    let timestamp = 1_700_000_000_i64;
    let location = Some("nyc_coords".to_string());

    let message = build_hardware_signature_message(&listing, &data_hash, timestamp, &location);
    let keypair = fixture_keypair();
    let signature = keypair.sign(&message);

    // Round-trip verification using the same primitives as the on-chain program.
    let public = DalekPublicKey::from_bytes(&keypair.public.to_bytes()).unwrap();
    assert!(public.verify(&message, &signature).is_ok());

    // Tampering with the message should invalidate the signature.
    let mut tampered = message.clone();
    tampered.push(42);
    assert!(public.verify(&tampered, &signature).is_err());
}

#[test]
fn signature_window_enforced() {
    // Helper checks that constants expose the stricter 5 minute SLA.
    assert!(HARDWARE_SIGNATURE_MAX_AGE <= 300);
}
