CREATE TABLE user_2fa
(
    user_id     int NOT NULL PRIMARY KEY REFERENCES users ( id ),
    secret      char(32) NOT NULL UNIQUE,
    email_reset bool NOT NULL
);
