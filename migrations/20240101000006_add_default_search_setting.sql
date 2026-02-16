-- Add default search setting: NULL = not selected, or one of: book, author, series, translator
ALTER TABLE user_settings
ADD COLUMN default_search VARCHAR(32) NULL
CHECK (default_search IS NULL OR default_search IN ('book', 'author', 'series', 'translator'));
