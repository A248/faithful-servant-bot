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

use sqlx::postgres::PgPool;
use crate::config::{Induction, IrcServer};
use eyre::Result;

type IrcClient = irc::client::Client;

#[derive(Debug)]
pub struct IrcBot {
    irc_client: irc::client::Client,
    connection_pool: PgPool
}

impl IrcBot {
    pub async fn new(config: IrcServer,
                     connection_pool: PgPool) -> Result<Self> {

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
            connection_pool,
            irc_client
        })
    }

    pub async fn start(self) -> Result<()> {
        log::info!("Connecting to IRC...");
        self.irc_client.identify()?;
        log::info!("Connected to IRC");

        Ok(())
    }
}
