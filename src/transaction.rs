use ::serde::{Deserialize, Serialize};
use chrono::{serde::ts_seconds, DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: Decimal,
    pub comment: String,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            amount: dec!(0),
            comment: String::new(),
            date: Utc::now(),
        }
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionToSubmit {
    pub amount: String,
    pub comment: String,
    pub date: String,
}

impl TransactionToSubmit {
    pub fn new() -> Self {
        Self {
            amount: String::new(),
            comment: String::new(),
            date: String::new(),
        }
    }
}

impl Default for TransactionToSubmit {
    fn default() -> Self {
        Self::new()
    }
}