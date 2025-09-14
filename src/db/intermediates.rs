// SPDX-FileCopyrightText: 2025 FreshlyBakedCake
//
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::{Value, from_value};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct ReturnedNotification {
    pub id: i32,
    pub dbus_notification_id: uuid::Uuid,
}

#[derive(Debug, Clone, FromRow)]
pub struct Count {
    pub count: u32,
}

fn from_json_or_str<'de, D, T>(deser: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let v = Value::deserialize(deser)?;
    match v {
        Value::Array(_) | Value::Object(_) => from_value::<T>(v).map_err(serde::de::Error::custom),
        Value::String(s) => serde_json::from_str::<T>(&s).map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("unexpected json type")),
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ParsedDBusNotification {
    pub id: Uuid,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub expire_timeout: i32,
    #[serde(deserialize_with = "from_json_or_str")]
    pub actions: Vec<String>,
    #[serde(deserialize_with = "from_json_or_str")]
    pub hints: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ParsedNotification {
    pub id: u32,
    pub dbus_notification: ParsedDBusNotification,
    pub read: bool,
    pub history: Vec<ParsedDBusNotification>,
}
