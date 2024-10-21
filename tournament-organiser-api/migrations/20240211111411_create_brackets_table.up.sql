-- Add up migration script here
-- NOTE: some user may be the creator of the bracket but this info is kinda
--       useless on it's own. Just declare who can manage the bracket in some
--       other table.
-- NOTE: JSONB because it's better json which takes more space but allows more
--       interesting queries. Takes more time to insert but that should not be
--       that much of a problem.
-- NOTE: for now, starting time, bracket format... are not that much of a
--       concern. Let's deal with those things later
CREATE TABLE brackets(
    id uuid NOT NULL DEFAULT gen_random_uuid (),
    PRIMARY KEY (id),
    name TEXT NOT NULL,
    matches JSONB NOT NULL,
    created_at timestamptz NOT NULL default current_timestamp
);
