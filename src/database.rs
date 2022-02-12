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

use sqlx::{PgPool, Row};
use eyre::Result;

/// Database access. Cloning this struct is cheap as it simply increments a reference counter
#[derive(Clone, Debug)]
pub struct Database {
    connection_pool: PgPool,
}

impl From<PgPool> for Database {
    fn from(connection_pool: PgPool) -> Self {
        Self {
            connection_pool
        }
    }
}

impl Database {
    pub async fn create_schema(&self) -> Result<()> {
        let mut connection = self.connection_pool.acquire().await?;

        sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS "users" (
          "id" BIGINT NOT NULL GENERATED BY DEFAULT AS IDENTITY,
          "discord_id" BIGINT,
          "irc_nickname" VARCHAR(64),
          CONSTRAINT "users_id_uniqueness" UNIQUE ("id"),
          CONSTRAINT "users_discord_id_uniqueness" UNIQUE ("discord_id"),
          CONSTRAINT "users_irc_nickname_uniqueness" UNIQUE ("irc_nickname")
        )
        "#).execute(&mut connection).await?;
        sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS "inducted" (
          "user" BIGINT NOT NULL,
          CONSTRAINT "inducted_user_uniqueness" UNIQUE ("id"),
          CONSTRAINT "inducted_user_validity"
            FOREIGN KEY ("user") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE
        )
        "#).execute(&mut connection).await?;
        sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS "messages" (
          "sent_by" BIGINT NOT NULL,
          "word_count" INT NOT NULL,
          "created" BIGINT NOT NULL,
          CONSTRAINT "messages_sent_by_validity"
            FOREIGN KEY ("sent_by") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE
        )
        "#).execute(&mut connection).await?;
        sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS "messages_created_index" ON "messages" ("created")
        "#).execute(&mut connection).await?;

        Ok(())
    }

    pub async fn record_message(&self,
                                user_identifier: UserIdentifier<'_>,
                                word_count: u32) -> Result<()> {
        let mut transaction = self.connection_pool.begin().await?;

        let user_id_row = match user_identifier {
            UserIdentifier::DiscordId(discord_id) => {
                let discord_id = discord_id as i64;
                sqlx::query(r#"
                INSERT INTO "users" ("discord_id") VALUES (?)
                  ON CONFLICT ("discord_id") DO NOTHING
                "#).bind(discord_id).execute(&mut transaction).await?;
                sqlx::query(r#"
                SELECT "id" FROM "users" WHERE "discord_id" = ?
                "#).bind(discord_id).fetch_one(&mut transaction).await?
            }
            UserIdentifier::IrcNickname(irc_nickname) => {
                sqlx::query(r#"
                INSERT INTO "users" ("irc_nickname") VALUES (?)
                  ON CONFLICT ("irc_nickname") DO NOTHING
                "#).bind(irc_nickname).execute(&mut transaction).await?;
                sqlx::query(r#"
                SELECT "id" FROM "users" WHERE "irc_nickname" = ?
                "#).bind(irc_nickname).fetch_one(&mut transaction).await?
            }
        };
        let user_id: i64 = user_id_row.try_get("id")?;

        use std::time::{SystemTime, UNIX_EPOCH};
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to obtain duration since unix epoch")
            .as_secs();

        sqlx::query(r#"
        INSERT INTO "messages" ("sent_by", "word_count", "created") VALUES (?, ?, ?)
        "#)
            .bind(user_id)
            .bind(word_count as i32)
            .bind(creation_time as i64)
            .execute(&mut transaction)
            .await?;
        Ok(())
    }
}

pub enum UserIdentifier<'n> {
    DiscordId(u64),
    IrcNickname(&'n str)
}
