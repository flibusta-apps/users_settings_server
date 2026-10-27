-- Seed the languages book_bot uses by default (ru, be, uk).
-- No-op for rows that already exist (e.g. on the production DB).
INSERT INTO languages (label, code) VALUES
    ('Русский', 'ru'),
    ('Беларуская', 'be'),
    ('Українська', 'uk')
ON CONFLICT (code) DO NOTHING;
