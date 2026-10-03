
use std::{collections::HashMap, fmt::Write as _};

use anyhow::Result;
use docstr::docstr;
use poise::serenity_prelude::{ContentModifier::Bold, Member, Mentionable, UserId};
use serde::{Deserialize, Serialize};

use crate::{Context, bet::{Play, PlayResult}, utils::*};

const DEFAULT_PROFILE: &'static str = "default";
const MAX_PROFILES: usize = 20;
const DEFAULT_FUND: u64 = 1000;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    money: u64,
    history: Vec<Play>,
    init: u64,
    max: u64,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: UserId,
    profiles: HashMap<String, Profile>,
    active: String,
    pub money: u64,
}

impl Profile {
    pub fn new(money: u64) -> Self {
        Self { money, history: Vec::new(), init: money, max: money }
    }
    
    pub fn money(&self) -> u64 { self.money }
    
    pub fn play(&mut self, play: Play) {
        match play.result() {
            PlayResult::Gain(n) => self.money += n,
            PlayResult::Loss(n) => self.money -= n,
        }
        self.history.push(play);
    }
}

impl Default for Profile { fn default() -> Self { Self::new(DEFAULT_FUND) } }

impl User {
    pub fn new(id: UserId) -> Self {
        let mut profiles = HashMap::new();
        
        profiles.insert(DEFAULT_PROFILE.to_string(), Profile::default());
        
        Self { id, profiles, money: 1000, active: DEFAULT_PROFILE.to_string() }
    }
    
    pub fn id(&self) -> UserId { self.id }
    pub fn active_profile(&self) -> &str { &self.active }
    pub fn active_ref(&self) -> &Profile { self.profiles.get(&self.active).unwrap() }
    pub fn active_mut(&mut self) -> &mut Profile { self.profiles.get_mut(&self.active).unwrap() }
}

#[poise::command(prefix_command, slash_command)]
pub async fn list(context: Context<'_>, user: Option<Member>) -> Result<()> {
    let id = user.map_or(context.author().id, |user| user.user.id);
    
    let mut s = String::new();
    
    context.data().user(id)?.lock_ref(|user| {
        docstr!(writeln! s
            /// User: {user}
            /// 
            /// Profiles:
            user = id.mention(),
        )?;
        
        user.profiles.keys().try_for_each(|profile| {
            write!(&mut s, "{}", profile)?;
            if profile == &user.active { write!(&mut s, "{}", Bold + " (Active)")?; }
            writeln!(&mut s)
        })
    })??;
    context.say(s).await?;
    
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn view(context: Context<'_>, user: Option<Member>, name: String) -> Result<()> {
    let id = user.map_or(context.author().id, |user| user.user.id);
    
    let mut s = String::new();
    let mut p = true;
    
    context.data().user(id)?.lock_ref(|user| {
        let Some(profile) = user.profiles.get(&name) else { p = false; return Ok(()); };
        
        docstr!(write! s
            /// User: {user} (Profile: {profile})
            /// Money: {money}
            user = id.mention(),
            profile = name,
            money = profile.money,
        )?;
        
        writeln!(&mut s, "todo…")
    })??;
    
    if p { context.say(s).await?; }
    else { context.private_reply("Profile does not exist!").await?; }
    
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn create(context: Context<'_>, name: String, initial_fund: Option<u64>) -> Result<()> {
    context.private_reply(context.data().user(context.author().id)?.lock_mut(|user| {
        if user.profiles.contains_key(&name) {
            "Profile already exists!"
        }
        else if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            "Invalid profile name! Only alphabets, numbers, underscore or dash are allowed!"
        }
        else if user.profiles.len() < MAX_PROFILES {
            user.profiles.insert(name, initial_fund.map_or_default(|i| Profile::new(i)));
            "Profile successfully created!"
        }
        else {
            "Maximum profile count already reached! Remove another profile first!"
        }
    })?).await
}

#[poise::command(prefix_command, slash_command)]
pub async fn remove(context: Context<'_>, name: String) -> Result<()> {
    context.private_reply(context.data().user(context.author().id)?.lock_mut(|user| {
        if user.profiles.len() < 2 {
            "Cannot remove the only one profile"
        }
        else if user.profiles.remove(&name).is_some() {
            user.active = user.profiles.keys().next().unwrap().clone();
            "Profile successfully removed!"
        }
        else {
            "Profile does not exist!"
        }
    })?).await
}

#[poise::command(prefix_command, slash_command)]
pub async fn switch(context: Context<'_>, profile: String) -> Result<()> {
    enum State { NotSwitched, Switched, NotFound }
    use State::*;
    
    context.private_reply(match context.data().user(context.author().id)?.lock_mut(|user| {
        if user.active == profile { NotSwitched }
        else if user.profiles.contains_key(&profile) { user.active = profile; Switched }
        else { NotFound }
    })? {
        Switched => "Switched profile successfully",
        NotSwitched => "Already using this profile",
        NotFound => "Profile not found",
    }).await
}
