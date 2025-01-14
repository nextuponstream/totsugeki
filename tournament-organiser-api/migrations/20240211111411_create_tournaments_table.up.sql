-- Add up migration script here
-- NOTE: some user may be the creator of the bracket but this info is kinda
--       useless on its own. Just declare who can manage the bracket in some
--       other table.
-- NOTE: JSONB because it's better json which takes more space but allows more
--       interesting queries. Takes more time to insert but that should not be
--       that much of a problem.
-- NOTE: for now, starting time, bracket format... are not that much of a
--       concern. Let's deal with those things later
CREATE TABLE tournaments
(
    id         uuid        NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    name       TEXT        NOT NULL,
    format     TEXT        NOT NULL
        CONSTRAINT tournament_format
            CHECK ( format IN ('double_elimination') ),
    created_at timestamptz NOT NULL default current_timestamp
);
