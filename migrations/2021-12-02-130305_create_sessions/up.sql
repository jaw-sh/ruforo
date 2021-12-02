CREATE TABLE sessions
(
    id         char(36) NOT NULL,
    data       json NOT NULL,
    expires_at timestamp NOT NULL,
    CONSTRAINT pk_session_id PRIMARY KEY ( id )
);
