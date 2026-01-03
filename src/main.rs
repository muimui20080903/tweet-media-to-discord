use dotenv::dotenv;
use std::env;

use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler, GatewayIntents};
use serenity::{async_trait, Client};

use regex::Regex;

use serde::Deserialize;

// vxtwitterAPI受信用の型
#[derive(Debug, Deserialize)]
pub struct TweetResponse {
    pub date: String,
    pub date_epoch: i64,

    #[serde(rename = "hasMedia")]
    pub has_media: bool,

    pub text: String,

    #[serde(rename = "mediaURLs")]
    pub media_urls: Vec<String>,

    #[serde(rename = "tweetURL")]
    pub tweet_url: String,

    pub user_name: String,
    pub user_screen_name: String,
    pub user_profile_image_url: String,
}

struct Handler; // discordのイベントハンドラー用

#[async_trait]
impl EventHandler for Handler {
    // messageが送られてきた際の処理
    async fn message(&self, ctx: Context, msg: Message) {
        // 発言者がbotの場合は中断
        if msg.author.bot {
            return;
        }

        // messageにtweetのurlが含まれていない場合は中断
        if let Some(hash) = match_twitter_url(&msg.content) {
            // vxtwitterのAPIにアクセスしてmediaを取得
            let response = get_from_vxtwitter_api(&hash)
                .await
                .expect("vxtwitterAPIerror");

            // mediaなしの場合中断
            if !response.has_media {
                return;
            }

            // responseからメッセージ(embed)とメディア(media_urls)を取り出す
            let (embed, attachments): (CreateEmbed, Vec<CreateAttachment>) =
                format_api_response_to_message(&ctx.http, response)
                    .await
                    .expect("メディア取り出しエラー");

            // メッセージを送る
            let embed_message_builder: CreateMessage = CreateMessage::new().add_embed(embed);
            let _ = msg
                .channel_id
                .send_message(&ctx.http, embed_message_builder)
                .await
                .expect("Error sending embed message");

            // 画像を送る
            let files_message_builder: CreateMessage = CreateMessage::new().content("");
            let _ = msg
                .channel_id
                .send_files(&ctx.http, attachments, files_message_builder)
                .await
                .expect("Error sending media files");
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn match_twitter_url(content: &str) -> Option<String> {
    // 正規表現を使ってハッシュを取り出す
    let regex = Regex::new(r"https://(x|twitter).com/([a-zA-Z0-9_]{1,16})/status/(?<hash>[0-9]+)")
        .expect("Failed to create regex");

    // 正規表現にマッチした場合はハッシュを返す
    regex.captures(content).map(|caps| caps["hash"].to_string())
}

async fn get_from_vxtwitter_api(hash: &String) -> Result<TweetResponse, reqwest::Error> {
    // vxtwitterのAPIにアクセスする
    // user_agentはcurl
    let client: reqwest::Client = reqwest::Client::builder()
        .user_agent("curl/7.81.0")
        .build()?;
    // 末尾にtweet_id
    let url: String = format!("https://api.vxtwitter.com/Twitter/status/{}", hash);
    // GET
    let response: reqwest::Response = client
        .get(url)
        .header("Accept", "*/*")
        .send()
        .await
        .expect("http getの失敗");
    let response_str: String = response.text().await?;
    // 自分で型定義したTweetResponseにパース
    let response_obj: TweetResponse = serde_json::from_str(&response_str).expect("json変換の失敗");
    Ok(response_obj)
}

async fn format_api_response_to_message(
    http: &Http,
    res: TweetResponse,
) -> Result<(CreateEmbed, Vec<CreateAttachment>), serenity::Error> {
    let embed_author: CreateEmbedAuthor = CreateEmbedAuthor::new("embed_media_author")
        .name(format!("{}({})", &res.user_name, &res.user_screen_name))
        .icon_url(&res.user_profile_image_url);

    let embed_timestamp: serenity::model::Timestamp =
        serenity::model::Timestamp::from_unix_timestamp(res.date_epoch).unwrap();

    let embed: CreateEmbed = CreateEmbed::new()
        .author(embed_author) // 作成者
        // .title(format!("{}({})", &res.user_screen_name, &res.user_name))
        .url(&res.tweet_url)
        .timestamp(embed_timestamp)
        .description(&res.text);

    let mut attachments = Vec::new();

    for url in &res.media_urls {
        let attachment = CreateAttachment::url(http, url).await?;
        attachments.push(attachment);
    }

    Ok((embed, attachments))
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn improper_url() {
        // textにtwitterのリンクが含まれていた場合tweet_idを取り出す
        let sample_text = String::from("https://x.com/elonmusk/status/1349129669258448897");
        if let Some(hash) = match_twitter_url(&sample_text) {
            assert_eq!(hash, String::from("1349129669258448897"));
        } else {
            panic!("{}のパースに失敗", sample_text);
        }
    }
}
