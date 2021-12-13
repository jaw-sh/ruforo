-- ************************************** user_name

CREATE TABLE user_name
(
    id         serial NOT NULL PRIMARY KEY,
    user_id    int NOT NULL REFERENCES users ( id ),
    name       text NOT NULL,
    created_at timestamp NOT NULL,
    is_public  bool NOT NULL DEFAULT true
);

CREATE INDEX ON user_name ( user_id );
CREATE INDEX ON user_name ( user_id, created_at DESC );
CREATE INDEX ON user_name ( user_id, is_public );

-- Import user names from the user table.
INSERT INTO user_name (user_id, name, created_at) SELECT id, name, created_at FROM users;

-- ************************************** ugc_deletions

CREATE TABLE ugc_deletions
(
    id         int NOT NULL PRIMARY KEY REFERENCES ugc ( id ),
    user_id    int NULL REFERENCES users ( id ),
    name       text NULL,
    deleted_at timestamp NOT NULL,
    reason     text NULL
);

-- ************************************** users
DROP TYPE IF EXISTS CONTENT_STATUS;
CREATE TYPE CONTENT_STATUS AS ENUM ('visible', 'hidden', 'in_queue');

ALTER TABLE ugc ADD COLUMN content_status CONTENT_STATUS NOT NULL DEFAULT 'visible'::CONTENT_STATUS;
