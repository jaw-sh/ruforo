-- ************************************** users
CREATE TYPE PASSWORD_CIPHER AS ENUM ('argon2id', 'bcrypt');

CREATE TABLE users
(
    id          SERIAL NOT NULL,
    created_at  TIMESTAMP NOT NULL,
    name        TEXT NOT NULL UNIQUE,
    password    TEXT NOT NULL,
    password_cipher PASSWORD_CIPHER NOT NULL,
    CONSTRAINT pk_user_id PRIMARY KEY ( id ),
    CONSTRAINT ak_user_name UNIQUE ( name )
);
