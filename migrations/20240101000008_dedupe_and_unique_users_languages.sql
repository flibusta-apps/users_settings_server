-- Deduplicate existing users_languages rows (keep lowest id per user+language pair)
DELETE FROM users_languages a
USING users_languages b
WHERE a.id > b.id
  AND a."user" = b."user"
  AND a.language = b.language;

-- Prevent future duplicates at the database level
ALTER TABLE users_languages
ADD CONSTRAINT uq_users_languages_user_language UNIQUE ("user", language);
