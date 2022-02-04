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
mod irc;
mod discord;

use async_std::path::PathBuf;
use async_std::task;
use eyre::Result;
use crate::discord::DiscordBot;
use crate::irc::IrcBot;

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

    let Config { postgres_url, irc_server, discord_bot, induction } = Config::load(
        &PathBuf::from("config.ron")
    ).await?;

    let connection_pool = sqlx::postgres::PgPool::connect_lazy(&postgres_url)?;
    log::info!("Connected to database");

    let irc_task = {
        let connection_pool = connection_pool.clone();
        task::spawn(async move {
            let irc_bot = IrcBot::new(irc_server, connection_pool).await?;
            irc_bot.start().await
        })
    };
    let discord_task = {
        let discord_bot = DiscordBot::new(discord_bot, connection_pool);
        task::spawn(async move {
            discord_bot.start().await
        })
    };
    irc_task.await
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
