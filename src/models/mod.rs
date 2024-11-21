// src/models/mod.rs

use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CheckItem {
    pub checked: bool,
    pub children: IndexMap<String, CheckItem>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Document {
    pub name: String,
    pub checklist: IndexMap<String, CheckItem>,
    pub last_modified: DateTime<Utc>,
}

impl CheckItem {
    pub fn new() -> Self {
        Self {
            checked: false,
            children: IndexMap::new(),
        }
    }
}