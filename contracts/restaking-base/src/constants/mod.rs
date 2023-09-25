use near_sdk::{Balance, EpochHeight};
use near_units::parse_near;

pub mod gas_constants;

pub const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 4;
pub const STORAGE_FEE: Balance = parse_near!("0.01 near");

pub const REGISTER_STORAGE_FEE: Balance = parse_near!("0.02 near");
