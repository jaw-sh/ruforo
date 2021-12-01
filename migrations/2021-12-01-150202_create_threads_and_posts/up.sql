-- ************************************** threads

CREATE TABLE threads
(
    id serial NOT NULL,
    title     text NOT NULL,
    subtitle  text NULL,
    CONSTRAINT pk_thread_id PRIMARY KEY ( id )
);


-- ************************************** posts

CREATE TABLE posts
(
    id        serial NOT NULL,
    thread_id serial NOT NULL,
    ugc_id    serial NOT NULL,
    post_id   serial NOT NULL,
    CONSTRAINT pk_post_id PRIMARY KEY ( post_id ),
    CONSTRAINT fk_post_thread_id FOREIGN KEY ( thread_id ) REFERENCES threads ( id ),
    CONSTRAINT fk_post_ugc_id FOREIGN KEY ( ugc_id ) REFERENCES ugc ( id )
);

CREATE INDEX index_post_thread_id ON posts ( thread_id );
CREATE INDEX index_post_ugc_id ON posts ( ugc_id );
