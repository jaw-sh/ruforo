-- ************************************** ugc

CREATE TABLE ugc
(
    id                serial PRIMARY KEY,
    ugc_revision_id   int NULL
);

-- ************************************** ugc_revisions

CREATE TABLE ugc_revisions
(
    id              serial PRIMARY KEY,
    ugc_id          int NOT NULL REFERENCES ugc,
    ip_id           int NULL REFERENCES ip,
    user_id         int NULL REFERENCES users,
    created_at      timestamp NOT NULL,
    content         text NOT NULL
);

-- **************************************

ALTER TABLE ugc ADD CONSTRAINT fk_ugc_ugc_revision_id FOREIGN KEY ( ugc_revision_id ) REFERENCES ugc_revisions ( id );

CREATE INDEX index_ugc_revision_ugc_history ON ugc_revisions ( ugc_id, created_at );
