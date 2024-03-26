-- Add up migration script here
ALTER TABLE brackets
ADD participants JSONB NOT NULL;
