-- Add up migration script here
CREATE TABLE users(
    id uuid NOT NULL DEFAULT gen_random_uuid (),
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at timestamptz NOT NULL default current_timestamp
);
