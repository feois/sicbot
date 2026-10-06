
use std::sync::{Mutex, RwLock};

use anyhow::{Result, bail};
use poise::{CreateReply, ReplyHandle, serenity_prelude::CreateAllowedMentions};

use crate::Context;

pub trait LockRef<T> { fn lock_ref<U>(&self, f: impl FnOnce(&T) -> U) -> Result<U>; }
pub trait LockMut<T>: LockRef<T> { fn lock_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> Result<U>; }

impl<T> LockMut<T> for Mutex<T> {
    fn lock_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> Result<U> {
        let Ok(mut t) = self.lock() else { bail!("Mutex poisoned") };
        Ok(f(&mut t))
    }
}

impl<T> LockRef<T> for Mutex<T> {
    fn lock_ref<U>(&self, f: impl FnOnce(&T) -> U) -> Result<U> {
        self.lock_mut(|t| f(&*t))
    }
}

impl<T> LockRef<T> for RwLock<T> {
    fn lock_ref<U>(&self, f: impl FnOnce(&T) -> U) -> Result<U> {
        let Ok(t) = self.read() else { bail!("Lock poisoned") };
        Ok(f(&t))
    }
}

impl<T> LockMut<T> for RwLock<T> {
    fn lock_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> Result<U> {
        let Ok(mut t) = self.write() else { bail!("Lock poisoned") };
        Ok(f(&mut t))
    }
}

pub struct ReplyMessage<'a, 'b, 'c> {
    context: &'b Context<'a>,
    private: bool,
    mention: bool,
    handle: Option<&'c mut ReplyHandle<'a>>,
}

impl<'a, 'b, 'c> ReplyMessage<'a, 'b, 'c> {
    fn new(context: &'b Context<'a>) -> Self { Self { context, private: false, mention: true, handle: None } }
    pub fn private(mut self) -> Self { self.private = true; self }
    pub fn visibility(self, private: bool) -> Self { if private { self.private() } else { self } }
    pub fn no_mention(mut self) -> Self { self.mention = false; self }
    pub fn handle(mut self, handle: &'c mut ReplyHandle<'a>) -> Self { self.handle = Some(handle); self }
    pub async fn send(self, message: impl Into<String>) -> Result<()> {
        let mut reply = CreateReply::new().content(message);
        
        if !self.mention { reply = reply.allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users()) }
        if self.private { reply = reply.ephemeral(true) }
        
        let handle = self.context.send(reply).await?;
        
        if let Some(h) = self.handle { *h = handle }
        
        Ok(())
    }
}

pub trait ContextExt { fn message(&self) -> ReplyMessage<'_, '_, '_>; }

impl<'a> ContextExt for Context<'a> {
    fn message(&self) -> ReplyMessage<'_, '_, '_> { ReplyMessage::new(self) }
}
