-- ************************************** user_names

CREATE TABLE user_names
(
    user_id    int NOT NULL PRIMARY KEY REFERENCES users ( id ),
    name       text NOT NULL UNIQUE
);

-- Import user names from the user table.
INSERT INTO user_names (user_id, name) SELECT id, name FROM users;

-- ************************************** user_name_history

CREATE TABLE user_name_history
(
    user_id     int NOT NULL PRIMARY KEY REFERENCES users ( id ),
    created_at  timestamp NOT NULL,
    approved_at timestamp NOT NULL,
    approved_by int NULL DEFAULT NULL REFERENCES users ( id ),
    name        text NOT NULL,
    reason      text NULL DEFAULT NULL,
    is_public   bool NOT NULL DEFAULT true
);

CREATE INDEX ON user_name_history ( user_id, approved_at DESC );

-- Import user names from the user table.
INSERT INTO user_name_history (user_id, created_at, approved_at, name)
    SELECT id, created_at, created_at, name FROM users;


-- Drop from users
ALTER TABLE users DROP COLUMN name;
