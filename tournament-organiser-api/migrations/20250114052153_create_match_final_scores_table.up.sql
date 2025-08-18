-- Add up migration script here
CREATE TABLE match_final_scores
(
    match_id               uuid REFERENCES matches (id),
    PRIMARY KEY (match_id),

    high_seed_player_score smallint
        CONSTRAINT bounded_high_seed_player_score
            CHECK (
                high_seed_player_score IS NULL
                    OR (0 <= high_seed_player_score)
                ),

    low_seed_player_score  smallint
        CONSTRAINT bounded_low_seed_player_score
            CHECK (
                low_seed_player_score IS NULL
                    OR (0 <= low_seed_player_score)
                ),

    CONSTRAINT final_score_sanity_check
        CHECK (
            NOT (high_seed_player_score = 0 AND low_seed_player_score = 0)
            ),

    -- NOTE: "PostgreSQL does not support CHECK constraints that reference table
    -- data other than the new or updated row being checked" (see https://www.postgresql.org/docs/current/ddl-constraints.html)
    -- This means the app HAS to assert a score report is consistent with
    -- match format.
    -- Example:
    -- 10-0 is not possible for first to 3

    winner                 uuid REFERENCES players (id),
    -- must assert that winner is a player in the match application side
    automatic_loser        uuid REFERENCES players (id)
        CONSTRAINT winner_is_not_loser CHECK ( winner IS NULL OR automatic_loser IS NULL ),
    -- must assert that automatic loser is a player in the match application side

    created_at             timestamptz NOT NULL default current_timestamp
)
