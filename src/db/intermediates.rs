use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct ReturnedNotification {
    pub id: i32,
    pub dbus_notification_id: uuid::Uuid,
}

#[derive(Debug, Clone, FromRow)]
pub struct Count {
    pub count: u32,
}
