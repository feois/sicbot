
use std::sync::{Mutex, RwLock};

use anyhow::{Result, bail};
use poise::CreateReply;

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

pub trait ContextUtils {
    async fn private_reply(&self, s: impl Into<String>) -> Result<()>;
}

impl ContextUtils for Context<'_> {
    async fn private_reply(&self, s: impl Into<String>) -> Result<()> {
        self.send(CreateReply::new().content(s).ephemeral(true)).await?;
        Ok(())
    }
}
