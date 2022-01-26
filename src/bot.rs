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

use irc::client::Client;
use sqlx::postgres::PgPool;
use crate::config::Induction;
use eyre::Result;

#[derive(Debug)]
pub struct Bot {
    pub connection_pool: PgPool,
    pub irc_client: Client,
    pub induction: Induction
}

impl Bot {
    pub async fn start(self) -> Result<()> {
        self.irc_client.identify()?;
        log::info!("Connected to IRC");

        Ok(())
    }
}
