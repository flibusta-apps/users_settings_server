-- Activity has no consumers of its surrogate id. Replace its existing unique
-- constraint with the primary key; DROP COLUMN deliberately does not use CASCADE,
-- so an unexpected external FK/dependency aborts rather than being destroyed.
ALTER TABLE user_activity DROP COLUMN id;
ALTER TABLE user_activity
    DROP CONSTRAINT user_activity_user_key,
    ADD CONSTRAINT user_activity_pkey PRIMARY KEY ("user");

-- Settings ids are referenced by both tables. Preserve every value and the
-- sequence's current position, widening the entire FK graph before restoring
-- the original referential actions. SQLx runs this migration transactionally.
ALTER TABLE user_activity
    DROP CONSTRAINT fk_user_activity_user_settings_id_user;
ALTER TABLE users_languages
    DROP CONSTRAINT fk_users_languages_user_settings_user_id;

ALTER TABLE user_settings ALTER COLUMN id TYPE BIGINT;
ALTER SEQUENCE user_settings_id_seq AS BIGINT;
ALTER TABLE user_activity ALTER COLUMN "user" TYPE BIGINT;
ALTER TABLE users_languages ALTER COLUMN "user" TYPE BIGINT;

-- Language-association inserts also consume sequence values on conflicts.
-- Retain existing ids and sequence position while removing the int4 ceiling.
ALTER TABLE users_languages ALTER COLUMN id TYPE BIGINT;
ALTER SEQUENCE users_languages_id_seq AS BIGINT;

ALTER TABLE user_activity
    ADD CONSTRAINT fk_user_activity_user_settings_id_user
    FOREIGN KEY ("user") REFERENCES user_settings(id);
ALTER TABLE users_languages
    ADD CONSTRAINT fk_users_languages_user_settings_user_id
    FOREIGN KEY ("user") REFERENCES user_settings(id)
    ON UPDATE CASCADE ON DELETE CASCADE;
