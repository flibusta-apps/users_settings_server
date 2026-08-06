-- Migrate naive timestamps to TIMESTAMPTZ (see docs/specs/02-migrations-schema-gaps.md #02.4).
--
-- Existing naive values in both columns are assumed to already represent UTC instants
-- (user_activity.updated is written via SQL NOW(); chat_donate_notifications.sended was
-- written by the app via chrono::offset::Local::now().naive_local(), which in every known
-- deployment runs in a UTC container). The USING clause below makes that assumption explicit
-- when reinterpreting existing rows as timestamptz.
ALTER TABLE user_activity
ALTER COLUMN updated TYPE TIMESTAMPTZ USING updated AT TIME ZONE 'UTC';

ALTER TABLE chat_donate_notifications
ALTER COLUMN sended TYPE TIMESTAMPTZ USING sended AT TIME ZONE 'UTC';
