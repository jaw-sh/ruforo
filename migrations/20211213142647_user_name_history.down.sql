-- Add down migration script here
ALTER TABLE users ADD COLUMN name text NULL;

UPDATE users SET name=n.name FROM user_names n WHERE id=n.user_id;

ALTER TABLE users ALTER COLUMN name SET NOT NULL;

ALTER TABLE users ADD CONSTRAINT unique_user_name UNIQUE (name);

DROP TABLE IF EXISTS user_names;
DROP TABLE IF EXISTS user_name_history;
