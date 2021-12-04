-- ************************************** threads

CREATE TABLE threads
(
    id serial     NOT NULL,
    user_id       int NULL,
    created_at    timestamp NOT NULL,
    title         text NOT NULL,
    subtitle      text NULL,
    post_count    int NOT NULL,
    first_post_id int NULL,
    last_post_id  int NULL,
    last_post_at  timestamp NULL,
    CONSTRAINT pk_thread_id PRIMARY KEY ( id ),
    CONSTRAINT fk_thread_user_id FOREIGN KEY ( user_id ) REFERENCES users ( id ),
    CONSTRAINT fk_thread_first_post_id FOREIGN KEY ( first_post_id ) REFERENCES threads ( id ),
    CONSTRAINT fk_thread_last_post_id FOREIGN KEY ( last_post_id ) REFERENCES threads ( id )
);

CREATE INDEX index_thread_user_id ON threads ( user_id );
CREATE INDEX index_thread_last_post_at ON threads ( last_post_at DESC);


-- ************************************** posts

CREATE TABLE posts
(
    id        serial NOT NULL,
    thread_id int NOT NULL,
    position  int NOT NULL,
    ugc_id    int NOT NULL,
    user_id   int NULL,
    created_at timestamp NOT NULL,
    CONSTRAINT pk_post_id PRIMARY KEY ( id ),
    CONSTRAINT fk_post_thread_id FOREIGN KEY ( thread_id ) REFERENCES threads ( id ),
    CONSTRAINT fk_post_ugc_id FOREIGN KEY ( ugc_id ) REFERENCES ugc ( id ),
    CONSTRAINT fk_post_user_id FOREIGN KEY ( user_id ) REFERENCES users ( id )
);

CREATE INDEX index_post_thread_id ON posts ( thread_id );
CREATE INDEX index_post_ugc_id ON posts ( ugc_id );
CREATE INDEX index_post_user_id ON posts ( user_id );
