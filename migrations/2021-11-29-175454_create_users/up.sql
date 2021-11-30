-- ************************************** tf_users

CREATE TABLE tf_users
(
 user_id    serial NOT NULL,
 created_on timestamp NULL,
 name       text NOT NULL,
 CONSTRAINT pk_user_id PRIMARY KEY ( user_id )
);
