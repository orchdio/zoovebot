// TODO: implement running at interval
// TODO: properly handle errors, for example duplicate tweet errors and (probably) rate limit errors
// TODO: unit tests and remove some values hidden behind env variables

extern crate dotenv;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use dotenv::dotenv;
use egg_mode::tweet as eTweet;
use egg_mode::*;
use preview_rs::Preview;
use redis::Commands;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::cell::RefCell;
use std::env;
use std::fmt::Display;
use std::time::Duration as StdDuration;

use tokio::{select, spawn, task::spawn_blocking, time::interval};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    pub data: Data,
    pub message: String,
    pub status: i64,
}

impl Display for APIResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({})", self)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub entity: String,
    pub platforms: Platforms,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platforms {
    pub deezer: Option<Deezer>,
    pub spotify: Option<Spotify>,
    pub tidal: Option<Tidal>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deezer {
    pub url: Option<String>,
    pub artistes: Option<Vec<String>>,
    pub released: Option<String>,
    pub duration: Option<String>,
    pub explicit: bool,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub album: Option<String>,
    pub id: Option<String>,
    pub cover: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spotify {
    pub url: Option<String>,
    pub artistes: Option<Vec<String>>,
    pub released: Option<String>,
    pub duration: Option<String>,
    pub explicit: bool,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub album: Option<String>,
    pub id: Option<String>,
    pub cover: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tidal {
    pub url: Option<String>,
    pub artistes: Option<Vec<String>>,
    pub released: Option<String>,
    pub duration: Option<String>,
    pub explicit: bool,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub album: Option<String>,
    pub id: Option<String>,
    pub cover: Option<String>,
}

#[tokio::main]
async fn main() {
    let ticker1 = ticker(15);
    let ticker = spawn_blocking(|| ticker1).await.unwrap();
    select! {
        _ = ticker => {
            println!("ticker finished");
        }
    }
}

async fn ticker(seconds: u64) {
    let mut interval = interval(StdDuration::from_secs(seconds));
    loop {
        interval.tick().await;
        function().await;
    }
}

async fn function() {
    dotenv().ok();
    // implement an async code that fetches last mentions from twitter api using reqwest
    // and then prints them to the console
    let interval_time =
        env::var("ZOOVEBOT_INTERVAL_TIME").expect("ZOOVEBOT_INTERVAL_TIME must be set");

    let consumer_key = env::var("ZOOVEBOT_APIKEY").expect("ZOOVEBOT_APIKEY must be set");
    let consumer_secret =
        env::var("ZOOVEBOT_APIKEY_SECRET").expect("ZOOVEBOT_APIKEY_SECRET must be set");

    let access_key = env::var("ZOOVEBOT_ACCESS_TOKEN").expect("ZOOVEBOT_ACCESS_KEY must be set");
    let access_secret =
        env::var("ZOOVEBOT_ACCESS_TOKEN_SECRET").expect("ZOOVEBOT_ACCESS_TOKEN_SECRET must be set");
    let orchdio_endpoint =
        env::var("ZOOVEBOT_ORCHDIO_ENDPOINT").expect("ZOOVEBOT_ORCHDIO_ENDPOINT must be set");

    // connect to redis
    let client = redis::Client::open(
        env::var("ZOOVEBOT_REDIS_URL").expect("ZOOVEBOT_REDIS_URL must be set"),
    )
    .unwrap();

    let mut con = client.get_connection().expect("Failed to connect to redis");

    let consumer_token = KeyPair::new(consumer_key, consumer_secret);
    let access_token = KeyPair::new(access_key, access_secret);
    let token = Token::Access {
        access: access_token,
        consumer: consumer_token,
    };

    let (_timeline, feed) = tweet::mentions_timeline(&token)
        .with_page_size(100)
        .start()
        .await
        .expect("Failed to get mentions");

    for tweet in feed {
        let created_at = tweet.created_at.to_rfc2822();
        if Utc::now()
            .checked_sub_signed(Duration::minutes(interval_time.parse::<_>().unwrap()))
            .unwrap()
            <= DateTime::parse_from_rfc2822(&created_at).unwrap()
        {
            // check if tweet is already processed
            let tweet_id = tweet.id;
            let tweet_id_exists: bool = con.get(tweet_id.to_string()).unwrap_or(false);

            if tweet_id_exists {
                println!("Tweet already processed");
                continue;
            }

            println!(
                "The tweet has one link attached. Fetching the link now {:#?}",
                tweet.entities.urls
            );

            if tweet.clone().entities.urls.into_iter().count() > 1 {
                // TODO: implement replying to the tweet with the error message being printed to terminal
                println!("The tweet has more than one link attached. please send one link and make sure its a valid link on a streaming platform");
            }

            let tweet2 = tweet.clone();
            let link = tweet2.entities.urls[0]
                .expanded_url
                .as_ref()
                .unwrap()
                .to_string();

            // then get the preview.
            // let preview = get_preview(&link.to_owned()).await;
            let link = async {
                let p = Preview::async_new(&link.to_owned()).await;
                let url = p.fetch_preview().url;
                return url;
            }
            .await
            .unwrap_or_default();

            let api_response = reqwest::Client::new()
                .get(format!("{}={}", orchdio_endpoint, link))
                .header(
                    env::var("ORCHDIO_HEADER").expect("ORCHDIO_HEADER must be set"),
                    env::var("ZOOVEBOT_ORCHDIO_KEY").expect("ZOOVEBOT_ORCHDIO_KEY must be set"),
                )
                .send()
                .await
                .expect("FATAL: error converting link")
                .json::<APIResponse>()
                .await
                .expect("FATAL: error getting deserializing API response");

            let mut links = vec![
                api_response
                    .data
                    .platforms
                    .deezer
                    .unwrap_or_default()
                    .url
                    .unwrap_or_default(),
                api_response
                    .data
                    .platforms
                    .spotify
                    .unwrap_or_default()
                    .url
                    .unwrap_or_default(),
                api_response
                    .data
                    .platforms
                    .tidal
                    .unwrap_or_default()
                    .url
                    .unwrap_or_default()
                    .to_string(),
            ];

            // remove urls that may be empty
            links.retain(|x| x != "");

            let reply_text = format!(
                "Hey 👋🏾 @{}, here are some of the links i found for you:\n {}.\n
Please tag again to convert another track and I'll reply in a few minutes.",
                tweet.user.as_ref().unwrap().screen_name,
                links.join("\n ").trim()
            );

            let reply = eTweet::DraftTweet::new(reply_text.clone())
                .in_reply_to(tweet.id)
                .send(&token.clone())
                .await
                .expect("Failed to send reply");

            println!(
                "Replied to tweet {} with tweet that has ID {}",
                tweet.id, reply.id
            );

            // save the tweet id to redis
            let _: () = con.set(tweet.id, true).unwrap();

            // TODO: implement calling zoove api to get the links and then reply to the tweet
            println!(
                "{}: {} at {}. Replied wuth {}",
                tweet.user.as_ref().unwrap().screen_name,
                tweet.text,
                tweet.response.created_at,
                reply.text
            );
        }
    }

    println!("No mention yet. Checked at {}", Utc::now());
}
