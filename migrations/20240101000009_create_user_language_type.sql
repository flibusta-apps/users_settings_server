-- Create composite type user_language_type used by queries in src/views/users/mod.rs
-- (ARRAY_AGG(ROW(...)::user_language_type)) and decoded via
-- #[sqlx(type_name = "user_language_type")] in src/views/users/serializers.rs.
-- Guarded so it is a no-op on the legacy production DB where the type
-- already exists manually.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'user_language_type'
    ) THEN
        CREATE TYPE user_language_type AS (
            id INTEGER,
            label VARCHAR(16),
            code VARCHAR(4)
        );
    END IF;
END $$;
