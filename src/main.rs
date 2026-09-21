
use std::{collections::HashMap, env, sync::{Arc, RwLock}};

use anyhow::{Error, Result};
use poise::{CreateReply, serenity_prelude::{self as serenity, FutureExt}};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

type Context<'a> = poise::Context<'a, Arc<RwLock<data::Data>>, Error>;

mod data;
mod user;
mod bet;

const DATA_PATH: &'static str = "data.json";

async fn private_reply(context: Context<'_>, s: impl Into<String>) -> Result<()> {
    context.send(CreateReply::new().content(s).ephemeral(true)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, subcommands(
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
))]
async fn sicbo(_: Context<'_>) -> Result<()> { Ok(()) }

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
    
    let data = Arc::new(RwLock::new(std::fs::read_to_string(DATA_PATH).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| data::Data {
            users: HashMap::new(),
        })));
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
        
        if let Ok(data) = data.read().inspect_err(|_| eprintln!("Lock poisoned")) {
            std::fs::write(DATA_PATH, serde_json::to_string_pretty(&*data).unwrap()).unwrap();
        }
    });
    
    client.start().await.unwrap();
}
