-- Add up migration script here
CREATE TABLE users(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at timestamp NOT NULL
);
