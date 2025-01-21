-- Add up migration script here
CREATE TABLE tournament_matches
(
    tournament_id uuid     NOT NULL references tournaments (id),
    match_id      uuid     NOT NULL references matches (id),
    pos           smallint NOT NULL CHECK ( pos > 0 )
)
