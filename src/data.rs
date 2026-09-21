
use std::collections::HashMap;

use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Serialize, Deserialize, Default)]
pub struct Data {
    pub users: HashMap<UserId, User>,
}

impl Data {
    pub fn get(&self, id: UserId) -> Option<&User> { self.users.get(&id) }
    pub fn get_mut(&mut self, id: UserId) -> &mut User { self.users.entry(id).or_insert_with(User::new) }
}
