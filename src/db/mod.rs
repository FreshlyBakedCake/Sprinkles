// SPDX-FileCopyrightText: 2025 FreshlyBakedCake
//
// SPDX-License-Identifier: MIT

use intermediates::ParsedNotification;
use sqlx::{Connection, Row};
use std::collections::HashMap;
use std::str::FromStr as _;

pub mod intermediates;
pub mod parser;
pub mod tables;

pub struct DB {
    connection: sqlx::sqlite::SqliteConnection,
}

impl DB {
    pub async fn default() -> Self {
        let dev_mode = std::env::var("SPRINKLES_DEV").is_ok();
        let mut data_dir = dirs::data_local_dir().unwrap();
        data_dir.push("sprinkles");
        if !dev_mode && !std::fs::exists(data_dir.clone()).unwrap() {
            std::fs::create_dir(data_dir.clone()).unwrap();
        }
        let db_opts = sqlx::sqlite::SqliteConnectOptions::from_str(if dev_mode {
            "test.db"
        } else {
            data_dir.push("sprinkles.db");
            data_dir.to_str().unwrap()
        })
        .unwrap()
        .create_if_missing(true);
        DB::new(
            sqlx::SqliteConnection::connect_with(&db_opts)
                .await
                .unwrap(),
        )
        .await
    }

    pub async fn new(mut connection: sqlx::sqlite::SqliteConnection) -> Self {
        sqlx::migrate!().run(&mut connection).await.unwrap();
        DB { connection }
    }

    pub async fn get_parsed_notification(
        &mut self,
        notification_id: u32,
    ) -> Result<ParsedNotification, sqlx::Error> {
        let row = sqlx::query(include_str!("get_parsed_notification.sql"))
            .bind(notification_id)
            .fetch_one(&mut self.connection)
            .await?;

        let id = row.get("id");
        let read = row.get("read");
        let dbus_json_bytes: Vec<u8> = row.get("dbus_notification");
        let history_json_bytes: Vec<u8> = row.get("history");

        let dbus_json = String::from_utf8(dbus_json_bytes)
            .map_err(|a| sqlx::Error::Decode(format!("invalid utf-8: {a:?}").into()))?;
        let history_json = String::from_utf8(history_json_bytes)
            .map_err(|a| sqlx::Error::Decode(format!("invalid utf-8: {a:?}").into()))?;

        Ok(ParsedNotification {
            id,
            read,
            dbus_notification: serde_json::from_str(&dbus_json)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            history: serde_json::from_str(&history_json)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }

    pub async fn get_notification(
        &mut self,
        where_sql: &str,
        params: &Vec<String>,
    ) -> Result<Vec<intermediates::ParsedNotification>, sqlx::Error> {
        let sql = format!("SELECT * FROM notifications WHERE {where_sql}");
        let mut builder = sqlx::query_as(&sql);

        for param in params {
            builder = builder.bind(param);
        }

        let fetched_notifications: Vec<tables::Notification> =
            builder.fetch_all(&mut self.connection).await.unwrap();

        let mut notifications = Vec::<ParsedNotification>::new();

        for nf in fetched_notifications {
            let parsed = self.get_parsed_notification(nf.id).await.unwrap();
            notifications.push(parsed);
        }
        Ok(notifications)
    }

    pub async fn create_notification(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> Result<tables::Notification, sqlx::Error> {
        let mut tx = self.connection.begin().await?;
        let dbus_notification: tables::DBusNotification = sqlx::query_as(
            "INSERT INTO dbus_notifications
            (id, app_name, replaces_id, app_icon, summary, body, expire_timeout)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            ",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(app_name)
        .bind(if replaces_id == 0 {
            None
        } else {
            Some(replaces_id)
        })
        .bind(app_icon)
        .bind(summary)
        .bind(body)
        .bind(expire_timeout)
        .fetch_one(&mut *tx)
        .await?;

        for item in actions {
            sqlx::query(
                "
                INSERT INTO notification_actions
                (id, dbus_notification_id, action)
                VALUES (?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(dbus_notification.id)
            .bind(item)
            .execute(&mut *tx)
            .await?;
        }

        for (key, value) in hints {
            sqlx::query(
                "
                INSERT INTO notification_hints
                (id, dbus_notification_id, k, value)
                VALUES (?, ?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(dbus_notification.id)
            .bind(key)
            .bind(value.to_string())
            .execute(&mut *tx)
            .await?;
        }
        let count = sqlx::query_as::<_, intermediates::Count>(
            "
                        SELECT COUNT(id) as count FROM notifications
                        ",
        )
        .fetch_one(&mut *tx)
        .await?
        .count;

        let notification: tables::Notification = if replaces_id == 0
            || count == 0
            || replaces_id > count
        {
            sqlx::query_as(
                "
                INSERT INTO notifications
                (dbus_notification_id)
                VALUES (?)
                RETURNING *",
            )
            .bind(dbus_notification.id)
            .fetch_one(&mut *tx)
            .await?
        } else {
            let n_returned: intermediates::ReturnedNotification = sqlx::query_as(
                "
                SELECT id, dbus_notification_id FROM notifications WHERE id = ?
                ",
            )
            .bind(replaces_id)
            .fetch_one(&mut *tx)
            .await?;

            let prev_num = sqlx::query_as::<_, intermediates::Count>(
                "
                        SELECT COUNT(id) as count FROM previous_dbus_notifications
                        WHERE notification_id = ?
                    ",
            )
            .bind(n_returned.id)
            .fetch_one(&mut *tx)
            .await?
            .count;

            sqlx::query(
                    "
                INSERT INTO previous_dbus_notifications (id, dbus_notification_id, notification_id, idx)
                VALUES (?, ?, ?, ?)",
                )
                .bind(uuid::Uuid::new_v4())
                .bind(n_returned.dbus_notification_id)
                .bind(n_returned.id)
                .bind(prev_num)
                .execute(&mut *tx)
                .await?;
            sqlx::query_as(
                "
                UPDATE notifications
                SET dbus_notification_id = ?
                WHERE id = ?
                RETURNING *",
            )
            .bind(dbus_notification.id)
            .bind(replaces_id)
            .fetch_one(&mut *tx)
            .await?
        };
        tx.commit().await?;

        Ok(notification)
    }

    pub async fn close_notification(&mut self, notification_id: u32) {
        sqlx::query("UPDATE notifications SET read = true WHERE id = ?")
            .bind(notification_id)
            .execute(&mut self.connection)
            .await
            .unwrap();
    }
}
