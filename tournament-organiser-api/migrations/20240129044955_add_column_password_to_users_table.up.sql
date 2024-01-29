-- Add up migration script here
ALTER TABLE users
ADD password TEXT NOT NULL;
