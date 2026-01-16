-- Create user_settings table
CREATE TABLE IF NOT EXISTS user_settings (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE,
    last_name VARCHAR(64) NOT NULL,
    first_name VARCHAR(64) NOT NULL,
    username VARCHAR(32) NOT NULL,
    source VARCHAR(32) NOT NULL
);

-- Create unique index on user_id (if not exists from UNIQUE constraint)
CREATE UNIQUE INDEX IF NOT EXISTS user_settings_user_id_key ON user_settings(user_id);
