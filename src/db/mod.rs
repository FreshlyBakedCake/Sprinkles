use std::collections::HashMap;

pub mod intermediates;
pub mod tables;

pub struct DB {
    connection: sqlx::sqlite::SqliteConnection,
}

impl DB {
    pub fn new(connection: sqlx::sqlite::SqliteConnection) -> Self {
        DB { connection }
    }

    pub async fn get_notification(&mut self, id: u32) -> Result<tables::Notification, sqlx::Error> {
        sqlx::query_as("SELECT * FROM notifications WHERE id = ?")
            .bind(id)
            .fetch_one(&mut self.connection)
            .await
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
        .fetch_one(&mut self.connection)
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
            .execute(&mut self.connection)
            .await?;
        }

        for (key, value) in hints {
            sqlx::query(
                "
                INSERT INTO notification_hints
                (id, dbus_notification_id, hint, value)
                VALUES (?, ?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(dbus_notification.id)
            .bind(key)
            .bind(value.to_string())
            .execute(&mut self.connection)
            .await?;
        }

        let notification: tables::Notification = if replaces_id == 0 {
            sqlx::query_as(
                "
                INSERT INTO notifications
                (dbus_notification_id)
                VALUES (?)
                RETURNING *",
            )
            .bind(dbus_notification.id)
            .fetch_one(&mut self.connection)
            .await?
        } else {
            let n_returned: intermediates::ReturnedNotification = sqlx::query_as(
                "
                SELECT id, dbus_notification_id FROM notifications WHERE id = ?
                ",
            )
            .bind(replaces_id)
            .fetch_one(&mut self.connection)
            .await?;
            sqlx::query(
                "
                INSERT INTO previous_dbus_notifications (id, dbus_notification_id, notification_id)
                VALUES (?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(n_returned.dbus_notification_id)
            .bind(n_returned.id)
            .execute(&mut self.connection)
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
            .fetch_one(&mut self.connection)
            .await?
        };

        Ok(notification)
    }

    pub async fn close_notification(&mut self, notification_id: u32) {
        sqlx::query("UPDATE notifications SET closed = true WHERE id = ?")
            .bind(notification_id)
            .execute(&mut self.connection)
            .await
            .unwrap();
    }
}
