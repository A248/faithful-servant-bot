/*
 * faithful-servant-bot
 * Copyright © 2022 Anand Beh
 *
 * faithful-servant-bot is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * faithful-servant-bot is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with faithful-servant-bot. If not, see <https://www.gnu.org/licenses/>
 * and navigate to version 3 of the GNU General Public License.
 */

use std::sync::Arc;
use eyre::Result;
use futures::StreamExt;
use irc::client::ClientStream;
use irc::proto::{Command, Prefix};
use crate::database::{Database, UserIdentifier};
use crate::ShutdownSignal;

type IrcConfig = crate::config::IrcServer;
type IrcClient = irc::client::Client;

#[derive(Debug)]
pub struct IrcBot {
    irc_client: irc::client::Client,
    database: Database
}

impl IrcBot {
    pub async fn new(config: IrcConfig,
                     database: Database) -> Result<Self> {

        let irc_client = IrcClient::from_config(irc::client::data::Config {
            owners: config.bot_owners,
            nickname: Some(config.bot_username),
            nick_password: Some(config.bot_password),
            server: Some(config.host),
            port: Some(config.port),
            use_tls: Some(true),
            encoding: Some(String::from("UTF-8")),
            channels: config.bot_channels,
            ..irc::client::data::Config::default()
        }).await?;
        Ok(IrcBot {
            irc_client,
            database,
        })
    }

    pub async fn start(mut self, shutdown_signal: Arc<ShutdownSignal>) -> Result<()> {
        log::info!("Connecting to IRC...");
        self.irc_client.identify()?;

        let message_stream = self.irc_client.stream()?;
        let irc_client = Arc::new(self.irc_client);

        let reception_future = MessageReceiver {
            message_stream,
            database: self.database,
            irc_client: irc_client.clone()
        }.receive_messages();
        let shutdown_future = async move {
            shutdown_signal.await_shutdown().await;
            irc_client.send_quit("Goodbye")
        };

        let (r1, r2) = futures::future::join(reception_future, shutdown_future).await;
        r1?;
        r2?;
        Ok(())
    }
}

#[derive(Debug)]
struct MessageReceiver {
    message_stream: ClientStream,
    database: Database,
    irc_client: Arc<IrcClient>
}

impl MessageReceiver {
    async fn receive_messages(mut self) -> Result<()> {

        while let Some(irc_message) = self.message_stream.next().await.transpose()? {

            if let Some(Prefix::Nickname(nickname, _, _)) = irc_message.prefix {

                // 1. Respond to the message
                // 2. Record the message

                let content = match irc_message.command {
                    Command::PRIVMSG(target, content) => {
                        // Respond only to PRIVMSG per the IRC protocol
                        // NOTICE commands should not be responded to
                        if let Some(response) = crate::brain::respond_to_message(&content) {
                            self.irc_client.send(Command::NOTICE(target, response.into_owned()))?;
                        }
                        content
                    },
                    Command::NOTICE(_, content) => content,
                    _ => continue
                };
                let message_handle = MessageHandle {
                    database: self.database.clone(),
                    nickname,
                    content
                };
                async_std::task::spawn(async move {
                    if let Err(e) = message_handle.handle().await {
                        log::error!("Error handling IRC message: {}", e)
                    }
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MessageHandle {
    database: Database,
    nickname: String,
    content: String
}

impl MessageHandle {
    async fn handle(self) -> Result<()> {
        self.database.record_message(
            UserIdentifier::IrcNickname(&self.nickname),
            crate::brain::count_words(self.content)
        ).await
    }
}

