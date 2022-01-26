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

#![forbid(unsafe_code)]

mod config;
mod bot;

use async_std::path::PathBuf;
use eyre::Result;
use crate::bot::Bot;

fn main() -> core::result::Result<(), eyre::Report> {
    use std::env;

    if let Err(env::VarError::NotPresent) = env::var("RUST_BACKTRACE") {
        env::set_var("RUST_BACKTRACE", "1");
        println!("Enabled RUST_BACKTRACE");
    }
    color_eyre::install()?;
    async_std::task::block_on(async_main())
}

async fn async_main() -> Result<()> {
    use crate::config::Config;
    use irc::client;

    let Config { postgres_url, irc_server, induction } = Config::load(
        &PathBuf::from("config.ron")
    ).await?;

    let connection_pool = sqlx::postgres::PgPool::connect_lazy(&postgres_url)?;
    log::info!("Connected to database");

    let irc_client = client::Client::from_config(client::data::Config {
        owners: irc_server.bot_owners,
        nickname: Some(irc_server.bot_username),
        nick_password: Some(irc_server.bot_password),
        server: Some(irc_server.host),
        port: Some(irc_server.port),
        use_tls: Some(true),
        encoding: Some(String::from("UTF-8")),
        channels: irc_server.bot_channels,
        ..client::data::Config::default()
    }).await?;
    log::info!("Validated bot details. Connecting to IRC...");

    let bot = Bot {
        connection_pool,
        irc_client,
        induction
    };
    bot.start().await
}

#[cfg(test)]
pub mod test_util {
    use tempfile::TempDir;
    use async_std::path::PathBuf;

    pub fn temp_file_in(tempdir: &TempDir, name: &str) -> PathBuf {
        let mut path = PathBuf::from(
            tempdir.path().to_path_buf().into_os_string()
        );
        path.push(name);
        path
    }
}
