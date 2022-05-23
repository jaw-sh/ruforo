CREATE TYPE GROUP_TYPE AS ENUM ('normal', 'system_guest', 'system_anon', 'system_user');
CREATE TYPE PERMISSION_FLAG AS ENUM ('yes', 'no', 'default', 'never');

CREATE TABLE groups
(
    id serial NOT NULL PRIMARY KEY,
    label text NOT NULL,
    group_type GROUP_TYPE NOT NULL DEFAULT 'normal'::GROUP_TYPE
);

CREATE TABLE user_groups
(
    user_id integer NOT NULL REFERENCES users ( id ) ON DELETE CASCADE,
    group_id integer NOT NULL REFERENCES groups ( id ) ON DELETE CASCADE,
    PRIMARY KEY (user_id, group_id)
);

CREATE INDEX ON user_groups ( user_id );
CREATE INDEX ON user_groups ( group_id );

CREATE TABLE permission_categories
(
    id serial NOT NULL PRIMARY KEY,
    label text NOT NULL,
    sort integer NOT NULL DEFAULT 0
);

CREATE TABLE permissions
(
    id serial NOT NULL PRIMARY KEY,
    category_id integer NOT NULL REFERENCES permission_categories ( id ) ON DELETE CASCADE,
    label text NOT NULL,
    sort integer NOT NULL DEFAULT 0
);

CREATE INDEX ON permissions ( category_id );

CREATE TABLE permission_collections
(
    id serial NOT NULL PRIMARY KEY,
    group_id integer REFERENCES groups ( id ) ON DELETE CASCADE,
    user_id integer REFERENCES users ( id ) ON DELETE CASCADE,
    UNIQUE (group_id, user_id)
);

CREATE INDEX ON permission_collections ( group_id );
CREATE INDEX ON permission_collections ( user_id );


CREATE TABLE permission_values
(
    permission_id integer NOT NULL REFERENCES permissions ( id ) ON DELETE CASCADE,
    collection_id integer NOT NULL REFERENCES permission_collections ( id ) ON DELETE CASCADE,
    value PERMISSION_FLAG NOT NULL DEFAULT 'default'::PERMISSION_FLAG,
    PRIMARY KEY (permission_id, collection_id)
);

CREATE INDEX ON permission_values ( permission_id );
CREATE INDEX ON permission_values ( collection_id );

CREATE TABLE forum_permissions
(
    forum_id integer NOT NULL REFERENCES forums ( id ) ON DELETE CASCADE,
    collection_id integer NOT NULL REFERENCES permission_collections ( id ) ON DELETE CASCADE,
    PRIMARY KEY (forum_id, collection_id)
);

CREATE INDEX ON forum_permissions ( forum_id );
CREATE INDEX ON forum_permissions ( collection_id );
