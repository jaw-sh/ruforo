-- ************************************** tf_users

CREATE TABLE users
(
    id         SERIAL NOT NULL,
    created_at TIMESTAMP NOT NULL,
    name       TEXT NOT NULL UNIQUE,
    password   TEXT NOT NULL,
    CONSTRAINT pk_user_id PRIMARY KEY ( id )
);
