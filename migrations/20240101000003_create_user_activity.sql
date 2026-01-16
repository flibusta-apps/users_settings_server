-- Create user_activity table
CREATE TABLE IF NOT EXISTS user_activity (
    id SERIAL PRIMARY KEY,
    "user" INTEGER NOT NULL UNIQUE,
    updated TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- Create unique index on user (if not exists from UNIQUE constraint)
CREATE UNIQUE INDEX IF NOT EXISTS user_activity_user_key ON user_activity("user");

-- Add foreign key constraint if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_user_activity_user_settings_id_user'
    ) THEN
        ALTER TABLE user_activity
        ADD CONSTRAINT fk_user_activity_user_settings_id_user
        FOREIGN KEY ("user") REFERENCES user_settings(id);
    END IF;
END $$;
