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
use serenity::client::{Context, EventHandler};
use async_trait::async_trait;
use eyre::Result;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use crate::database::{Database, UserIdentifier};
use crate::ShutdownSignal;

type DiscordClient = serenity::client::Client;

#[derive(Debug)]
pub struct DiscordBot {
    config: crate::config::DiscordBot,
    database: Database
}

impl DiscordBot {
    pub fn new(config: crate::config::DiscordBot, database: Database) -> Self {
        Self {
            config,
            database
        }
    }

    pub async fn start(self, shutdown_signal: Arc<ShutdownSignal>) -> Result<()> {

        let mut client = DiscordClient::builder(self.config.bot_token)
            .event_handler(Handler {
                database: self.database
            })
            .await?;

        let shard_manager = client.shard_manager.clone();

        let start_task = client.start();
        let shutdown_task = async move {
            shutdown_signal.await_shutdown().await;
            shard_manager.lock().await.shutdown_all().await;
            Ok::<_, eyre::Report>(())
        };

        let (r1, r2) = futures::future::join(start_task, shutdown_task).await;
        r1?;
        r2?;
        Ok(())
    }
}

#[derive(Debug)]
struct Handler {
    database: Database
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, new_message: Message) {

        let UserId(discord_id) = new_message.author.id;
        let word_count = crate::brain::count_words(new_message.content);

        self.database.record_message(
            UserIdentifier::DiscordId(discord_id),
            word_count
        ).await.expect("Failed to record message");
    }
}
