-- ************************************** tf_ip

CREATE TABLE tf_ip
(
 ip_id         serial NOT NULL,
 ip            inet NOT NULL,
 first_seen_on timestamp NOT NULL,
 last_seen_on  timestamp NOT NULL,
 CONSTRAINT pk_ip_id PRIMARY KEY ( ip_id ),
 CONSTRAINT ak_ip_inet UNIQUE ( ip )
);
