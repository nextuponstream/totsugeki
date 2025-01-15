-- Add up migration script here
-- when there is only 1 or two players, then no matches are saved. Still, we
-- need to persist the info that 1 player has joined the tournament
CREATE TABLE players
(
    tournament_id uuid NOT NULL REFERENCES tournaments (id),
    player_id     uuid NOT NULL REFERENCES users (id),
    PRIMARY KEY (tournament_id, player_id)
);
