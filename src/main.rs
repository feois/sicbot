
use std::{env, sync::Arc};

use anyhow::{Error, Result};
use docstr::docstr;
use poise::{serenity_prelude::{self as serenity, FutureExt}};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

use crate::{data::Data, utils::ContextUtils};

type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

mod data;
mod user;
mod bet;
mod room;
mod utils;

const DATA_PATH: &'static str = "data.json";

#[poise::command(slash_command, prefix_command, subcommands(
    "help",
    "bet::small",
    "bet::big",
    "bet::odd",
    "bet::even",
    "bet::any",
    "bet::any2",
    "bet::any3",
    "bet::any21",
    "bet::double",
    "bet::triple",
    "bet::all",
    "bet::total",
    "bet::four",
    "bet::rules",
    "user::create",
    "user::remove",
    "user::switch",
    "user::list",
    "user::view",
))]
async fn sicbo(_: Context<'_>) -> Result<()> { Ok(()) }

#[poise::command(prefix_command, slash_command)]
async fn help(context: Context<'_>) -> Result<()> {
    context.private_reply(docstr!(
        /// 
    )).await
}

async fn console() -> Result<()> {
    let mut reader = BufReader::new(stdin());
    let mut out = stdout();
    let mut line = String::new();

    loop {
        out.write_all(b"> ").await?;
        out.flush().await?;
        
        line.clear();
        
        if reader.read_line(&mut line).await? == 0 { break }

        match line.trim() {
            "exit" => break,
            _ => eprintln!("Unknown command"),
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env not found");
    
    let data = Arc::new(std::fs::read_to_string(DATA_PATH).ok()
        .and_then(|s| serde_json::from_str::<Data>(&s).ok())
        .unwrap_or_default());
    let data_clone = data.clone();
    let token = env::var("DISCORD_TOKEN").expect("envvar DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![sicbo()],
            ..Default::default()
        })
        .setup(|ctx, _, framework|
            Box::pin(poise::builtins::register_globally(ctx, &framework.options().commands)
                .map(|r| r.map(|()| data_clone).map_err(From::from))))
        .build();
    
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await.unwrap();
    
    let shard_manager = client.shard_manager.clone();
    
    tokio::task::spawn(async move {
        if let Err(e) = console().await { eprintln!("Console error: {}", e) }
        
        shard_manager.shutdown_all().await;
        
        std::fs::write(DATA_PATH, serde_json::to_string_pretty(&data).unwrap()).unwrap();
    });
    
    client.start().await.unwrap();
}
