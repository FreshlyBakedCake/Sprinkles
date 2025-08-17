-- Add migration script here
CREATE TABLE dbus_notifications (
    id STRING PRIMARY KEY,
    app_name TEXT NOT NULL,
    replaces_id INTEGER REFERENCES notifications (id),
    app_icon TEXT,
    summary TEXT NOT NULL,
    body TEXT,
    expire_timeout INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    dbus_notification_id STRING REFERENCES dbus_notifications (id),
    closed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE notification_actions (
    id STRING PRIMARY KEY,
    dbus_notification_id INTEGER REFERENCES notifications (id),
    action TEXT NOT NULL
);

CREATE TABLE notification_hints (
    id STRING PRIMARY KEY,
    dbus_notification_id STRING REFERENCES dbus_notifications (id),
    k TEXT NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE previous_dbus_notifications (
    id STRING PRIMARY KEY,
    dbus_notification_id STRING REFERENCES dbus_notifications (id),
    notification_id INTEGER REFERENCES notifications (id),
    idx INTEGER NOT NULL
);
