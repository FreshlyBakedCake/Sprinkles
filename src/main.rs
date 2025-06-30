#![feature(let_chains)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::{connection::Builder as ConnectionBuilder, interface as dbus_interface};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DBusNotification {
    app_name: String,
    replaces_id: u32,
    app_icon: String,
    summary: String,
    body: String,
    actions: Vec<String>,
    hints: HashMap<String, zbus::zvariant::OwnedValue>,
    expire_timeout: i32,
}

impl DBusNotification {
    pub fn new(
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> DBusNotification {
        Self {
            app_name: app_name.to_string(),
            replaces_id,
            app_icon: app_icon.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            actions: actions.iter().map(|action| action.to_string()).collect(),
            hints: hints
                .iter()
                .map(|(key, value)| (key.to_string(), value.try_to_owned().unwrap()))
                .collect(),
            expire_timeout,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Notification {
    dbus_notification: DBusNotification,
    id: u32,
    closed: bool,
}

impl Notification {
    pub fn new(id: u32, dbus_notification: DBusNotification) -> Notification {
        Self {
            dbus_notification,
            id,
            closed: false,
        }
    }
}

struct NotificationServer {
    last_id: u32,
    notifications: HashMap<u32, Notification>,
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
        let notification = DBusNotification::new(
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions.clone(),
            hints,
            expire_timeout,
        );
        let id: u32;
        if let Some(found_notification) = self.notifications.get_mut(&replaces_id)
            && app_name == found_notification.dbus_notification.app_name {
            id = replaces_id;
            *found_notification = Notification::new(id, notification);
        } else {
            id = self.last_id + 1;
            self.notifications
                .insert(id, Notification::new(id, notification));
            self.last_id = id;
        };
        self.notifications.iter().for_each(|(id, notification)| {
            if !notification.closed {
                println!("{id} {} - {}", notification.dbus_notification.app_name, notification.dbus_notification.summary);
            }
        });
        println!("--------");
        return id;
    }

    async fn close_notification(&mut self, id: u32) {
        if let Some(notification) = self.notifications.get_mut(&id) {
            notification.closed = true;
        }
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
    let notification_server = NotificationServer {
        last_id: 0,
        notifications: HashMap::new(),
    };

    let _conn = ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification_server)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}
