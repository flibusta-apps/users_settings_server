-- Create languages table
CREATE TABLE IF NOT EXISTS languages (
    id SERIAL PRIMARY KEY,
    label VARCHAR(16) NOT NULL,
    code VARCHAR(4) NOT NULL UNIQUE
);

-- Create unique index on code (if not exists from UNIQUE constraint)
CREATE UNIQUE INDEX IF NOT EXISTS languages_code_key ON languages(code);
