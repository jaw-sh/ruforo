-- ************************************** forums

CREATE TABLE forums
(
    id serial NOT NULL PRIMARY KEY,
    label text NOT NULL,
    description text NULL,
    last_post_id int NULL,
    last_thread_id int NULL
);

-- ************************************** threads

CREATE TABLE threads
(
    id            serial NOT NULL PRIMARY KEY,
    user_id       int NULL REFERENCES users ( id ) ON DELETE SET NULL,
    forum_id      int NOT NULL REFERENCES forums ( id ) ON DELETE CASCADE,
    created_at    timestamp NOT NULL,
    title         text NOT NULL,
    subtitle      text NULL,
    view_count    int NOT NULL,
    post_count    int NOT NULL,
    first_post_id int NULL,
    last_post_id  int NULL,
    last_post_at  timestamp NULL
);

CREATE INDEX ON threads ( forum_id );
CREATE INDEX ON threads ( user_id );
CREATE INDEX ON threads ( last_post_at DESC);


-- ************************************** posts

CREATE TABLE posts
(
    id        serial NOT NULL PRIMARY KEY,
    thread_id int NOT NULL REFERENCES threads ( id ) ON DELETE CASCADE,
    position  int NOT NULL,
    ugc_id    int NOT NULL REFERENCES ugc ( id ),
    user_id   int NULL REFERENCES users ( id ) ON DELETE SET NULL,
    created_at timestamp NOT NULL
);

CREATE INDEX ON posts ( thread_id );
CREATE INDEX ON posts ( ugc_id );
CREATE INDEX ON posts ( user_id );
CREATE INDEX ON posts ( position );

ALTER TABLE threads ADD CONSTRAINT threads_first_post_id_fkey FOREIGN KEY ( first_post_id ) REFERENCES posts ( id ) ON DELETE SET NULL;
ALTER TABLE threads ADD CONSTRAINT threads_last_post_id_fkey FOREIGN KEY ( last_post_id ) REFERENCES posts ( id ) ON DELETE SET NULL;

ALTER TABLE forums ADD CONSTRAINT forums_last_thread_id_fkey FOREIGN KEY ( last_thread_id ) REFERENCES threads ( id ) ON DELETE SET NULL;
ALTER TABLE forums ADD CONSTRAINT forums_last_post_id_fkey FOREIGN KEY ( last_post_id ) REFERENCES posts ( id ) ON DELETE SET NULL;