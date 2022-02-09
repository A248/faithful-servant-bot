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


use async_std::fs::{self, OpenOptions};
use async_std::path::Path;
use async_std::io::BufWriter;
use eyre::Result;
use ron::ser::PrettyConfig;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub postgres_url: String,
    pub irc_server: IrcServer,
    pub discord_bot: DiscordBot,
    pub induction: Induction
}

impl Config {
    pub async fn load(path: &Path) -> Result<Self> {
        use ron::ser;
        use async_std::io::WriteExt;

        if path.exists().await {
            let config = fs::read_to_string(path).await?;
            let config = ron::from_str(&config)?;
            Ok(config)
        } else {
            let config = Self::default();

            // Write default config
            let file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path).await?;
            let mut writer = BufWriter::new(file);
            writer.write_all(
                ser::to_string_pretty(&config, PrettyConfig::default())?.as_bytes()
            ).await?;
            writer.flush().await?;

            Ok(config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            postgres_url: String::from("postgres://user:password@host:port/database"),
            irc_server: IrcServer::default(),
            discord_bot: DiscordBot::default(),
            induction: Induction::default()
        }
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrcServer {
    pub host: String,
    pub port: u16,
    pub bot_username: String,
    pub bot_password: String,
    pub bot_owners: Vec<String>,
    pub bot_channels: Vec<String>
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordBot {
    pub bot_token: String,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Induction {
    pub message_requirements: Vec<MessageRequirement>,
    pub induction_cycle_days: u8
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRequirement {
    pub message_count: u8,
    pub word_count: u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn write_default_config() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = crate::test_util::temp_file_in(&tempdir, "config.ron");

        Config::load(&path).await?;
        Ok(())
    }

    #[async_std::test]
    async fn reload_default_config() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = crate::test_util::temp_file_in(&tempdir, "config.ron");

        let config = Config::load(&path).await?;
        let reloaded = Config::load(&path).await?;
        assert_eq!(config, reloaded);
        Ok(())
    }
}
