use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DBusNotification {
    pub id: Uuid,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub expire_timeout: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: u32,
    pub dbus_notification_id: Uuid,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationAction {
    pub id: Uuid,
    pub dbus_notification_id: Uuid,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationHint {
    pub id: Uuid,
    pub dbus_notification_id: Uuid,
    pub k: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PreviousDbusNotification {
    pub id: Uuid,
    pub notification_id: u32,
    pub dbus_notification_id: Uuid,
    pub idx: u32,
}
