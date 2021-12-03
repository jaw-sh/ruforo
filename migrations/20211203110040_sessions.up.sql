CREATE TABLE sessions
(
    id         char(36) NOT NULL,
    user_id    int NOT NULL,
    expires_at timestamp NOT NULL,
    CONSTRAINT pk_session_id PRIMARY KEY ( id ),
    CONSTRAINT fk_session_user_id FOREIGN KEY ( user_id ) REFERENCES users ( id )
);

CREATE INDEX index_session_user_id ON sessions ( user_id );
