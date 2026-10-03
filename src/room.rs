
use std::collections::HashMap;

use anyhow::Result;
use poise::{serenity_prelude::{ChannelId, UserId}};
use serde::{Deserialize, Serialize};

use crate::{Context, bet::Bets};

#[derive(Serialize, Deserialize)]
pub struct Room {
    id: ChannelId,
    users: HashMap<UserId, Option<(Bets, u64)>>,
    ready: usize,
}

impl Room {
    pub fn new(id: ChannelId) -> Self {
        Self { id, users: HashMap::new(), ready: 0 }
    }
    
    pub fn join(&mut self, id: UserId) {
        
    }
    
    pub fn leave(&mut self, id: UserId) -> bool {
        true
    }
    
    pub fn play(&mut self, user: UserId, bet: Bets, amount: u64) {
        
    }
    
    pub async fn update(&mut self, context: Context<'_>) -> Result<()> {
        if self.ready < self.users.len() { return Ok(()); }
        
        let channel = self.id.to_channel(context).await?.guild().unwrap();
        
        
        
        self.users.values_mut().for_each(|bets| *bets = None);
        self.ready = 0;
        
        Ok(())
    }
}
