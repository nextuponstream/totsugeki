-- Add up migration script here
CREATE TABLE users
(
    id         uuid        NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    email      TEXT        NOT NULL UNIQUE,
    -- I know about the practicality of asking for full name for very serious
    -- business. But for the sake of simplicity, we only ask the name.
    -- If this needs to evolve, then name should become gamer_tag. Also, you
    -- should ask for first name, last name and full name for completeness
    name       TEXT        NOT NULL,
    created_at timestamptz NOT NULL default current_timestamp
);
