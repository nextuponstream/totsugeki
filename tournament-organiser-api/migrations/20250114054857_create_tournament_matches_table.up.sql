-- Add up migration script here
CREATE TABLE tournament_matches
(
    tournament_id uuid references tournaments (id),
    match_id      uuid references matches (id),
    pos           integer CHECK ( pos > 0 )
)
