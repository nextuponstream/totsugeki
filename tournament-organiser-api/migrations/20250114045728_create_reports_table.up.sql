-- Add up migration script here
CREATE TABLE match_reports
(
    id                     uuid,
    PRIMARY KEY (id),
    match_id               uuid          NOT NULL REFERENCES matches (id),
    user_id                uuid          NOT NULL REFERENCES users (id),

    high_seed_player_score numeric(4, 0) NOT NULL
        CONSTRAINT positive_high_seed_player_score
            CHECK ( 0 <= high_seed_player_score ),

    low_seed_player_score  numeric(4, 0)
        CONSTRAINT positive_low_seed_player_score
            CHECK ( 0 <= low_seed_player_score ),

    created_at             timestamptz   NOT NULL default current_timestamp
)
