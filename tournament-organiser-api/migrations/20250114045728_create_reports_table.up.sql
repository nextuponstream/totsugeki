-- Add up migration script here
CREATE TABLE reports
(
    id                      uuid,
    PRIMARY KEY (id),
    match_id                uuid        NOT NULL REFERENCES matches (id),
    player_id               uuid REFERENCES players (id),
    tournament_organiser_id uuid REFERENCES tournament_organisers (id),

    CONSTRAINT either_player_or_tournament_organiser_report CHECK (
        player_id IS NOT NULL OR tournament_organiser_id IS NOT NULL
        ),
    high_seed_player_score  smallint    NOT NULL
        CONSTRAINT positive_high_seed_player_score
            CHECK ( 0 <= high_seed_player_score ),

    low_seed_player_score   smallint    NOT NULL,
    CONSTRAINT positive_low_seed_player_score
        CHECK ( 0 <= low_seed_player_score ),

    created_at              timestamptz NOT NULL default current_timestamp
)
