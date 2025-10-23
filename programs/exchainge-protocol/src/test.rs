#![cfg(test)]

use crate::state::{LicenseType, VerifierType, BPS_DENOMINATOR, PLATFORM_FEE_BPS};

#[test]
fn verifier_type_defaults_to_metadata() {
    let default = VerifierType::default();
    assert_eq!(default, VerifierType::Metadata);
}

#[test]
fn license_type_defaults_to_mit() {
    let default = LicenseType::default();
    assert_eq!(default, LicenseType::MIT);
}

#[test]
fn platform_fee_calculation() {
    let price = 1_000_000_000;
    let expected_fee = (price * PLATFORM_FEE_BPS) / BPS_DENOMINATOR;
    assert_eq!(expected_fee, 50_000_000);

    let price = 100_000_000;
    let expected_fee = (price * PLATFORM_FEE_BPS) / BPS_DENOMINATOR;
    assert_eq!(expected_fee, 5_000_000);
}

#[test]
fn seller_revenue_calculation() {
    let price = 1_000_000_000;
    let platform_fee = (price * PLATFORM_FEE_BPS) / BPS_DENOMINATOR;
    let seller_revenue = price - platform_fee;

    assert_eq!(platform_fee, 50_000_000);
    assert_eq!(seller_revenue, 950_000_000);
}

#[test]
fn test_enums_serialization() {
    use anchor_lang::AnchorSerialize;

    let verifier = VerifierType::AiAgent;
    let mut buf = Vec::new();
    verifier.serialize(&mut buf).unwrap();
    assert!(!buf.is_empty());

    let license = LicenseType::Commercial;
    let mut buf = Vec::new();
    license.serialize(&mut buf).unwrap();
    assert!(!buf.is_empty());
}

// TODO: Integration tests needed for platform init, dataset registration,
// purchase flow, access verification, owner-only updates, and error cases.
