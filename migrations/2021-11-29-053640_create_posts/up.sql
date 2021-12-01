CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    post_date TIMESTAMP NOT NULL
);

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    password VARCHAR NOT NULL,
    join_date TIMESTAMP NOT NULL,
    email VARCHAR UNIQUE
);
