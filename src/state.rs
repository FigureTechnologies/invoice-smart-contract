use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub static CONFIG_KEY: &[u8] = b"config";

pub static INVOICE_KEY: &[u8] = b"invoice";

/// Configuration state for the restricted marker transfer contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    // The owner
    pub admin: Addr,
    // Receipient of payment
    pub recipient: Addr,
    // The marker supported
    pub denom: String,
    // The human-readable name
    pub business_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Invoice {
    // Unique identifier
    pub id: String,
    // Amount of payment expected
    pub amount: Uint128,
    // The human-readable description of what it's for
    pub description: Option<String>,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn get_invoice_storage(storage: &mut dyn Storage) -> Bucket<Invoice> {
    bucket(storage, INVOICE_KEY)
}

pub fn get_invoice_storage_read(storage: &dyn Storage) -> ReadonlyBucket<Invoice> {
    bucket_read(storage, INVOICE_KEY)
}
