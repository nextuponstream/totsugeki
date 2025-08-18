-- Add up migration script here
CREATE TABLE guests
(
    id         uuid        NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    name       TEXT        NOT NULL,
    created_at timestamptz NOT NULL default current_timestamp
);
