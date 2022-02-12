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
mod database;
mod brain;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use async_std::path::PathBuf;
use async_std::task::{self, JoinHandle};
use eyre::Result;
use futures::{StreamExt, future};
use crate::database::Database;
use crate::discord::DiscordBot;
use crate::irc::IrcBot;

fn main() -> core::result::Result<(), eyre::Report> {
    use std::env;

    if let Err(env::VarError::NotPresent) = env::var("RUST_BACKTRACE") {
        env::set_var("RUST_BACKTRACE", "1");
        println!("Enabled RUST_BACKTRACE");
    }
    color_eyre::install()?;
    task::block_on(async_main())
}

async fn async_main() -> Result<()> {
    use crate::config::Config;

    let Config { postgres_url, irc_server, discord_bot, induction } = Config::load(
        &PathBuf::from("config.ron")
    ).await?;

    let database = Database::from(
        sqlx::postgres::PgPool::connect_lazy(&postgres_url)?
    );
    database.create_schema().await?;
    log::info!("Database is connected and ready");

    let shutdown_signal = Arc::new(ShutdownSignal::default());

    let irc_task = {
        let database = database.clone();
        let shutdown_signal = shutdown_signal.clone();
        task::spawn(async move {
            let irc_bot = IrcBot::new(irc_server, database).await?;
            irc_bot.start(shutdown_signal).await
        })
    };
    let discord_task = {
        let discord_bot = DiscordBot::new(discord_bot, database);
        let shutdown_signal = shutdown_signal.clone();
        task::spawn(async move {
            discord_bot.start(shutdown_signal).await
        })
    };
    await_shutdown(irc_task, discord_task, shutdown_signal).await
}

/*
Shutdown logic
 */

async fn await_shutdown(irc_task: JoinHandle<Result<()>>,
                        discord_task: JoinHandle<Result<()>>,
                        shutdown_signal: Arc<ShutdownSignal>) -> Result<()> {
    use signal_hook_async_std::Signals;
    use signal_hook::consts::signal::*;

    let mut signals = Signals::new([SIGINT, SIGTERM, SIGHUP])?;
    let _signal = signals.next().await;

    shutdown_signal.commence_shutdown();
    log::info!("Initiating shutdown...");

    let (irc_result, discord_result) = future::join(irc_task, discord_task).await;
    if let Err(e) = irc_result {
        log::error!("Error in IRC task: {}", e);
    }
    if let Err(e) = discord_result {
        log::error!("Error in discord task: {}", e);
    }
    Ok(())
}

pub struct ShutdownSignal {
    shutdown: AtomicBool,
    sleep_duration: Duration
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self {
            shutdown: AtomicBool::default(),
            sleep_duration: Duration::from_secs(2)
        }
    }
}

impl ShutdownSignal {
    fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    pub async fn await_shutdown(&self) {
        while !self.is_shutdown() {
            task::sleep(self.sleep_duration).await;
        }
    }

    fn commence_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst)
    }
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
