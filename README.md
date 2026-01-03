# tweet-media-to-discord

discordに投稿されたtwitterのurlから、メディアを取得してdiscordに投稿するbot

## how to use

1. discordBotを作成し、tokenを発行する。  
参考：[Rustで作るdiscord bot入門編 (serenity使用)](https://zenn.dev/t4t5u0/articles/cd731e0293cf224cb4dc)

2. `.env`に`DISCORD_TOKEN`を登録して実行

## 参考リンク

### discrod用

- [serenity](https://github.com/serenity-rs/serenity/tree/current)  
- [serenity document](https://docs.rs/serenity/latest/serenity/index.html)  
使用したクレート

- [Serenity (Rust) でDiscord Botを開発するときに躓いたところ](https://watasuke.net/blog/article/discord-bot-by-rust-and-serenity/)  
embedの書き方を参考にした。

- [Rustで作るdiscord bot入門編 (serenity使用)](https://zenn.dev/t4t5u0/articles/cd731e0293cf224cb4dc)  
botの書き方及びdiscord側の設定を参考にした。

### tweetのメディア情報取得

- [vxtwitter](https://github.com/dylanpdx/BetterTwitFix/blob/main/api.md)  
tweetからメディアの情報を得るために使用した。  
githubの情報と実際のresponse内容に差異があったため、取得できる情報は実際にcurlで叩くことで確認した。

- [reqwest document](https://docs.rs/reqwest/latest/reqwest/index.html)  
apiを叩くために使用した。
- [RustでWeb APIを叩く](https://qiita.com/odayushin/items/0e2a5a3d047e6b08c811)  
レスポンスボディのパース方法を参考にした。
