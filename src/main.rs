use chrono::{Datelike, TimeZone, Timelike, Utc};
use dotenv;
use serenity::{CacheAndHttp, async_trait, model::{channel::Message, gateway::Ready, id::UserId}, prelude::*};
use std::env;

const PREFIX: &str = "!";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if !msg.content.starts_with(PREFIX) {
            return;
        }

        let command_body = &msg.content[PREFIX.len()..];
        let mut args: Vec<&str> = command_body.split(" ").collect();
        let command = args.remove(0).to_lowercase();

        if command == "yp" {
            if args.is_empty() {
                if let Err(why) = msg.channel_id.say(&ctx.http, year_progress(None).await).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            else if args[0] == "ping" {
                let time_taken = (Utc::now() - msg.timestamp).num_milliseconds();
                if let Err(why) = msg.channel_id.say(&ctx.http, format!("Pong! This message had a latency of {}ms.", time_taken)).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            else if args[0] == "help" {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Available commands:\n1. !yp\n2. !yp ping\n-> !yp help").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            else {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Unknown arguments. Try '!yp help' for more information.").await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }
}

async fn year_progress(opt: Option<&CacheAndHttp>) -> String {
    let today = Utc::now();
    let current_year = today.year();

    let year_start = chrono::Utc.ymd(current_year, 1, 1);
    let next_year_start = chrono::Utc.ymd(current_year + 1, 1, 1);

    // Hours elapsed till start of day 0000h
    let hours_elapsed_since_year_start = (today.date() - year_start).num_hours();
    let total_hours_in_year = (next_year_start - year_start).num_hours();

    // Adding today.time().hour() to get time elapsed after 0000h
    let percentage = (((hours_elapsed_since_year_start + today.time().hour() as i64) as f64 / total_hours_in_year as f64) * 100f64) as i64;

    // Construct DM
    let bar_length = 15;
    let fill_length = ((percentage as f64 / 100f64) * bar_length as f64) as i32;
    let mut filled = String::new();
    (0..fill_length).for_each(|_| filled.push_str("▓"));
    let progress_bar = format!("{:░<15}", filled);
    let direct_msg = format!("[{}] {}%", progress_bar, percentage);

    return match opt {
        None => direct_msg,
        Some(cache_and_http) => {
            let user_id = env::var("USER_ID").expect("Expected a token in the environment").parse::<u64>().unwrap();
            let user = UserId(user_id).to_user(&cache_and_http).await.unwrap();
            let msg = user.direct_message(&cache_and_http, |m| {
                m.content(direct_msg)
            }).await.unwrap();
            msg.content
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    //TODO: loop year_progress() to send dm regularly
    /* let cache_and_http = client.cache_and_http.clone();
    tokio::spawn(async move {
        year_progress(Some(&cache_and_http)).await;
    }); */
    
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
