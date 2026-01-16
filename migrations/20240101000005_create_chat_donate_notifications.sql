-- Create chat_donate_notifications table
CREATE TABLE IF NOT EXISTS chat_donate_notifications (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL UNIQUE,
    sended TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- Create unique index on chat_id (if not exists from UNIQUE constraint)
CREATE UNIQUE INDEX IF NOT EXISTS chat_donate_notifications_chat_id_key ON chat_donate_notifications(chat_id);
