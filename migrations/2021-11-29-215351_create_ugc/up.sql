-- ************************************** tf_ugc

CREATE TABLE tf_ugc
(
    ugc_id          serial PRIMARY KEY,
    ugc_revision_id int NOT NULL
);

-- ************************************** tf_ugc_revisions

CREATE TABLE tf_ugc_revisions
(
    ugc_revision_id serial PRIMARY KEY,
    ugc_id          int NOT NULL REFERENCES tf_ugc,
    ip_id           int NULL REFERENCES tf_ip,
    user_id         int NULL REFERENCES tf_users,
    created_at      timestamp NOT NULL,
    content         text NULL
);

-- **************************************

ALTER TABLE tf_ugc ADD CONSTRAINT fk_ugc_ugc_revision_id FOREIGN KEY ( ugc_revision_id ) REFERENCES tf_ugc_revisions ( ugc_revision_id );

CREATE INDEX index_ugc_revision_ugc_history ON tf_ugc_revisions ( created_at, ugc_id );
