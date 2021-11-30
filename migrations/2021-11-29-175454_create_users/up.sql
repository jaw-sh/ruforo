-- ************************************** tf_users

CREATE TABLE users
(
    id    serial NOT NULL,
    created_at timestamp NULL,
    name       text NOT NULL,
    CONSTRAINT pk_user_id PRIMARY KEY ( id )
);
 
