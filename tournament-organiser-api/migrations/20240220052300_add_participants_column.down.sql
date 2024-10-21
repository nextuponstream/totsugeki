-- Add down migration script here
ALTER TABLE brackets
DROP COLUMN participants;
