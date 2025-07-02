#![feature(let_chains)]
use sqlx::Connection;
use std::str::FromStr as _;
use zbus::{connection::Builder as ConnectionBuilder, interface as dbus_interface};

mod db;

struct NotificationServer {
    database: db::DB,
}

#[dbus_interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let notification = self
            .database
            .create_notification(
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions,
                hints,
                expire_timeout,
            )
            .await
            .unwrap();
        println!("Notification created with ID: {}", notification.id);
        return notification.id;
    }

    async fn close_notification(&mut self, id: u32) {
        self.database.close_notification(id).await;
    }

    async fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "actions".to_string()]
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Sprinkles".to_string(),
            "FreshlyBakedCake".to_string(),
            "1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_opts = sqlx::sqlite::SqliteConnectOptions::from_str("test.db")
        .unwrap()
        .create_if_missing(true);

    let db = db::DB::new(sqlx::sqlite::SqliteConnection::connect_with(&db_opts).await?).await;

    let notification_server = NotificationServer { database: db };

    let _conn = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_server)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}
