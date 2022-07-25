-- Add up migration script here
CREATE TABLE IF NOT EXISTS chat_rooms
(
    id serial NOT NULL,
    title text NOT NULL,
    description text,
    display_order smallint NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX ON chat_rooms ( display_order );

CREATE TABLE IF NOT EXISTS chat_messages
(
    id serial NOT NULL,
    chat_room_id integer NOT NULL REFERENCES chat_rooms ( id ) ON DELETE CASCADE ON UPDATE CASCADE,
    ugc_id integer NOT NULL REFERENCES ugc ( id ) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id integer REFERENCES users ( id ) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at timestamp without time zone NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX ON chat_messages ( chat_room_id );
CREATE INDEX ON chat_messages ( ugc_id );
CREATE INDEX ON chat_messages ( user_id );
