-- ************************************** threads

CREATE TABLE threads
(
    id serial NOT NULL,
    user_id   int NULL,
    created_at timestamp NOT NULL,
    title     text NOT NULL,
    subtitle  text NULL,
    CONSTRAINT pk_thread_id PRIMARY KEY ( id ),
    CONSTRAINT fk_thread_user_id FOREIGN KEY ( user_id ) REFERENCES users ( id )
);

CREATE INDEX index_thread_user_id ON threads ( user_id );


-- ************************************** posts

CREATE TABLE posts
(
    id        serial NOT NULL,
    thread_id int NOT NULL,
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
