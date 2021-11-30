-- ************************************** tf_ip

CREATE TABLE ip
(
    id         serial NOT NULL,
    address    inet NOT NULL,
    first_seen_at timestamp NOT NULL,
    last_seen_at  timestamp NOT NULL,
    CONSTRAINT pk_ip_id PRIMARY KEY ( id ),
    CONSTRAINT ak_ip_inet UNIQUE ( address )
);
