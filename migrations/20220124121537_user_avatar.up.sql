CREATE TABLE user_avatars
(
    user_id       int NOT NULL PRIMARY KEY REFERENCES users ( id ) ON DELETE CASCADE,
    attachment_id int NOT NULL REFERENCES attachments ( id ) ON DELETE CASCADE,
    created_at    timestamp NOT NULL
);
