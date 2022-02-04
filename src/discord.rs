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

use std::collections::HashMap;
use serenity::client::{Context, EventHandler};
use sqlx::PgPool;
use async_trait::async_trait;
use eyre::Result;
use serenity::model::channel::Message;

type DiscordClient = serenity::client::Client;

#[derive(Debug)]
pub struct DiscordBot {
    config: crate::config::DiscordBot,
    connection_pool: PgPool
}

impl DiscordBot {
    pub fn new(config: crate::config::DiscordBot, connection_pool: PgPool) -> Self {
        Self {
            config,
            connection_pool
        }
    }

    pub async fn start(self) -> Result<()> {
        let mut client = DiscordClient::builder(self.config.bot_token)
            .event_handler(Handler {
                connection_pool: self.connection_pool
            })
            .await?;
        Ok(client.start().await?)
    }
}

#[derive(Debug)]
struct Handler {
    connection_pool: PgPool
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _new_message: Message) {
        todo!()
    }
}
