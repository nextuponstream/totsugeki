-- Add up migration script here
-- For now, we create matches for 2 players only.
-- If you need to use matches involving more people, then good luck with that.
-- The biggest offline tournament as of now has 7'000 players. Then 99'999
-- players limit is reasonable for now.
-- Lots of positive checks because postgresql does not provide rich type system.
-- FT100 is not unheard of. Then let's allow FT999.
CREATE TABLE matches
(
    id               uuid,
    PRIMARY KEY (id),

    high_seed        smallint    NOT NULL
        CONSTRAINT positive_high_seed
            CHECK ( high_seed > 0 ),
    high_seed_player uuid REFERENCES players (id),
    low_seed         smallint    NOT NULL
        CONSTRAINT seeds_are_different
            CHECK ( high_seed <> matches.low_seed )
        CONSTRAINT positive_low_seed
            CHECK ( low_seed > 0 ),
    low_seed_player  uuid
        CONSTRAINT players_are_different
            CHECK ( low_seed_player <> matches.high_seed_player )
        REFERENCES players (id),

    format           TEXT        NOT NULL
        CONSTRAINT match_format
            CHECK ( format IN ('first_to_n') ),
    format_n         smallint    NOT NULL
        CONSTRAINT positive_format_n
            CHECK ( format_n > 0 ),

    created_at       timestamptz NOT NULL default current_timestamp
);

CREATE INDEX high_seed_player_idx ON matches (high_seed_player);
CREATE INDEX low_seed_player_idx ON matches (low_seed_player);
