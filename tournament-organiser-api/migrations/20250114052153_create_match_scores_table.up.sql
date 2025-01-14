-- Add up migration script here
CREATE TABLE match_score
(
    match_id               uuid REFERENCES matches (id),
    report_id              uuid REFERENCES match_reports (id),

    high_seed_player_score numeric(4, 0)
        CONSTRAINT bounded_high_seed_player_score
            CHECK (
                high_seed_player_score IS NULL
                    OR (0 <= high_seed_player_score)
                ),

    low_seed_player_score  numeric(4, 0)
        CONSTRAINT bounded_low_seed_player_score
            CHECK (
                low_seed_player_score IS NULL
                    OR (0 <= low_seed_player_score)
                ),

    CONSTRAINT final_score_sanity_check
        CHECK (
            NOT (high_seed_player_score IS NULL AND low_seed_player_score IS NULL)
                AND NOT (high_seed_player_score = 0 AND low_seed_player_score = 0)
            )

    -- NOTE: "PostgreSQL does not support CHECK constraints that reference table
    -- data other than the new or updated row being checked" (see https://www.postgresql.org/docs/current/ddl-constraints.html)
    -- This means the app HAS to assert a score report is consistent with
    -- match format.
    -- Example:
    -- 10-0 is not possible for first to 3
)
