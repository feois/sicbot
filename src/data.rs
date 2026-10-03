
use std::{collections::HashMap, sync::{Arc, RwLock}};

use anyhow::Result;
use poise::serenity_prelude::{ChannelId, UserId};
use serde::{Deserialize, Serialize};

use crate::{Context, room::Room, user::User, utils::*};

#[derive(Serialize, Deserialize)]
pub struct Data {
    users: RwLock<HashMap<UserId, Arc<RwLock<User>>>>,
    rooms: RwLock<HashMap<ChannelId, Arc<RwLock<Room>>>>,
}

impl Default for Data {
    fn default() -> Self {
        Data { users: RwLock::new(HashMap::new()), rooms: RwLock::new(HashMap::new()) }
    }
}

impl Data {
    pub fn user(&self, id: UserId) -> Result<Arc<RwLock<User>>> {
        Ok(match self.users.lock_ref(|users| users.get(&id).cloned())? {
            Some(user) => user,
            None => self.users.lock_mut(|users|
                users.entry(id).or_insert(Arc::new(RwLock::new(User::new(id)))).clone())?,
        })
    }
    
    pub fn room(&self, id: ChannelId) -> Result<Option<Arc<RwLock<Room>>>> {
        self.rooms.lock_ref(|rooms| rooms.get(&id).cloned())
    }
    
    pub async fn join(&self, user: UserId, room: ChannelId, context: Context<'_>) -> Result<bool> {
        let room = match self.rooms.lock_ref(|rooms| rooms.get(&room).cloned())? {
            Some(room) => room,
            None => {
                if room.to_channel(context).await?.private().is_some() { return Ok(false) }
                
                self.rooms.lock_mut(|rooms|
                    rooms.entry(room).or_insert(Arc::new(RwLock::new(Room::new(room)))).clone())?
            }
        };
        
        room.lock_mut(|room| room.join(user))?;
        
        Ok(true)
    }
    
    pub fn leave(&self, user: UserId, room: ChannelId) -> Result<bool> {
        let Some(r) = self.room(room)? else { return Ok(false); };
        
        if r.lock_mut(|room| room.leave(user))? {
            self.rooms.lock_mut(|rooms| rooms.remove(&room))?;
        }
                
        Ok(true)
    }
}
