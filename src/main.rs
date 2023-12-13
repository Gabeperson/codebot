use codebot::run;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Configuration, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[group]
#[commands(syntax, eval, units)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("!"));

    // Login with a bot token from the environment
    let token = std::fs::read_to_string("token.txt").unwrap();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn syntax(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "https://numbat.dev/doc/example-numbat_syntax.html")
        .await?;

    Ok(())
}

#[command]
async fn eval(ctx: &Context, msg: &Message) -> CommandResult {
    use std::time::Instant;
    let now = Instant::now();
    let code = msg.content.replace("!eval", "");
    let code = code.trim();
    let l = code.len() - 1;

    if code.matches("```").count() != 2 || code.as_bytes()[l] != b'`' || code.as_bytes()[0] != b'`'
    {
        msg.reply(ctx, "Correct syntax for !eval is: \n!eval ``` <code> ```")
            .await?;
        return Ok(());
    };

    let code = code.replace("```", "").trim().to_string();

    let Ok(result) = tokio::task::spawn_blocking(|| run(code)).await else {
        msg.reply(ctx, "Something went wrong while running the file!")
            .await?;
        return Ok(());
    };
    let result = format!("Result: ```{}```", result);
    println!("{}ms elapsed", now.elapsed().as_micros() / 1000);
    msg.reply(ctx, result).await?;
    Ok(())
}

#[command]
async fn units(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "https://numbat.dev/doc/list-units.html")
        .await?;

    Ok(())
}
