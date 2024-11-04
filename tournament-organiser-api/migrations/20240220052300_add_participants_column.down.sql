-- Add down migration script here
ALTER TABLE tournaments
    DROP COLUMN participants;
