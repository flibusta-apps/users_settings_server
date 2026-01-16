-- Create users_languages table
CREATE TABLE IF NOT EXISTS users_languages (
    id SERIAL PRIMARY KEY,
    language INTEGER NOT NULL,
    "user" INTEGER NOT NULL
);

-- Add foreign key constraints if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_users_languages_languages_language_id'
    ) THEN
        ALTER TABLE users_languages
        ADD CONSTRAINT fk_users_languages_languages_language_id
        FOREIGN KEY (language) REFERENCES languages(id)
        ON UPDATE CASCADE ON DELETE CASCADE;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_users_languages_user_settings_user_id'
    ) THEN
        ALTER TABLE users_languages
        ADD CONSTRAINT fk_users_languages_user_settings_user_id
        FOREIGN KEY ("user") REFERENCES user_settings(id)
        ON UPDATE CASCADE ON DELETE CASCADE;
    END IF;
END $$;
