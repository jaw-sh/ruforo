-- Add down migration script here
DROP TABLE IF EXISTS attachments CASCADE;
DROP TABLE IF EXISTS ugc_attachments CASCADE;
DROP TABLE IF EXISTS attachment_thumbnails CASCADE;
