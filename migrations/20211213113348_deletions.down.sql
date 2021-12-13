-- Add down migration script here
DROP TABLE IF EXISTS ugc_deletions CASCADE;

ALTER TABLE ugc DROP COLUMN content_status;
DROP TYPE IF EXISTS CONTENT_STATUS;
