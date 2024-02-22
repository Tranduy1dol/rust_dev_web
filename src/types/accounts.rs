use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    pub exp: DateTime<Utc>,
    pub account_id: AccountId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub id: Option<AccountId>,
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub i32);
