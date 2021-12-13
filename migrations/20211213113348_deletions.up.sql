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
