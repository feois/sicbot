
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Profile {
    
}

#[derive(Serialize, Deserialize)]
pub struct User {
    profiles: HashMap<String, Profile>,
    pub money: u64,
}

impl User {
    pub fn new() -> Self {
        Self { profiles: HashMap::new(), money: 1000 }
    }
}
