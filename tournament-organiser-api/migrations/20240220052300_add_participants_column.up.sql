-- Add up migration script here
ALTER TABLE tournaments
    ADD participants JSONB NOT NULL;
