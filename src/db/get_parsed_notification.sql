-- SPDX-FileCopyrightText: 2025 FreshlyBakedCake
--
-- SPDX-License-Identifier: MIT

WITH nid AS (
    SELECT id, dbus_notification_id, closed
    FROM notifications
    WHERE id = ?
),
current AS (
    SELECT c.*, c.id AS raw_id
    FROM dbus_notifications c
    JOIN nid ON c.id = nid.dbus_notification_id
),
actions AS (
    SELECT action
    FROM notification_actions
    WHERE dbus_notification_id = (SELECT dbus_notification_id FROM nid)
    ORDER BY action
),
hints AS (
    SELECT k, value
    FROM notification_hints
    WHERE dbus_notification_id = (SELECT dbus_notification_id FROM nid)
),
history AS (
    SELECT
        h.*,
        p.idx,
        (
            SELECT json_group_array(action ORDER BY action)
            FROM notification_actions na
            WHERE na.dbus_notification_id = h.id
        ) AS actions,
        (
            SELECT json_group_object(k, value)
            FROM notification_hints nh
            WHERE nh.dbus_notification_id = h.id
        ) AS hints
    FROM previous_dbus_notifications p
    JOIN dbus_notifications h ON h.id = p.dbus_notification_id
    WHERE p.notification_id = (SELECT id FROM nid)
    ORDER BY p.idx ASC
)
SELECT
    (SELECT id          FROM nid)                                    AS id,
    (SELECT closed      FROM nid)                                    AS closed,
    (SELECT CAST(json_object(
        'id',            lower(substr(hex(current.id),1,8)||'-'||substr(hex(current.id),9,4)||'-'||substr(hex(current.id),13,4)||'-'||substr(hex(current.id),17,4)||'-'||substr(hex(current.id),21)),
        'app_name',      current.app_name,
        'replaces_id',   COALESCE(current.replaces_id, 0),
        'app_icon',      current.app_icon,
        'summary',       current.summary,
        'body',          current.body,
        'expire_timeout',current.expire_timeout,
        'actions',       (SELECT json_group_array(action) FROM actions),
        'hints',         (SELECT json_group_object(k,value) FROM hints)
    ) AS TEXT) FROM current)                                         AS dbus_notification,
    (SELECT CAST(json_group_array(
        json_object(
            'id',            lower(substr(hex(history.id),1,8)||'-'||substr(hex(history.id),9,4)||'-'||substr(hex(history.id),13,4)||'-'||substr(hex(history.id),17,4)||'-'||substr(hex(history.id),21)),
            'app_name',      history.app_name,
            'replaces_id',   COALESCE(history.replaces_id, 0),
            'app_icon',      history.app_icon,
            'summary',       history.summary,
            'body',          history.body,
            'expire_timeout',history.expire_timeout,
            'actions',       history.actions,
            'hints',         history.hints
        )
        ORDER BY idx
    ) AS TEXT) FROM history)                                        AS history
;
