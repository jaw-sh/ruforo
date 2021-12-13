-- ************************************** attachments

CREATE TABLE attachments
(
   id            serial NOT NULL PRIMARY KEY,
   filename      text NOT NULL,
   hash          char(64) NOT NULL,
   first_seen_at timestamp NOT NULL,
   last_seen_at  timestamp NOT NULL,
   banned_at     timestamp NULL,
   filesize      bigint NOT NULL,
   file_height   int NULL,
   file_width    int NULL,
   mime          text NOT NULL,
   meta          jsonb NOT NULL
);

-- ************************************** tf_attachment_thumbnails

CREATE TABLE attachment_thumbnails
(
    attachment_id int NOT NULL REFERENCES attachments ( id ),
    thumbnail_id  int NOT NULL REFERENCES attachments ( id ),
    PRIMARY KEY (attachment_id, thumbnail_id)
);

CREATE INDEX ON attachment_thumbnails ( attachment_id );
CREATE INDEX ON attachment_thumbnails ( thumbnail_id );

-- ************************************** ugc_attachments

CREATE TABLE ugc_attachments
(
    attachment_id int NOT NULL REFERENCES attachments ( id ),
    ugc_id        int NOT NULL REFERENCES ugc ( id ),
    user_id       int NULL REFERENCES users ( id ),
    ip_id         int NULL REFERENCES ip (id ),
    created_at    timestamp NOT NULL,
    filename      text NOT NULL,
    PRIMARY KEY (attachment_id, ugc_id)
);

CREATE INDEX ON ugc_attachments ( ugc_id );
CREATE INDEX ON ugc_attachments ( attachment_id );
CREATE INDEX ON ugc_attachments ( user_id );
CREATE INDEX ON ugc_attachments ( ip_id );
