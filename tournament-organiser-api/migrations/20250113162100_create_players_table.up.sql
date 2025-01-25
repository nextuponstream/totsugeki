-- Add up migration script here
-- when there is only 1 or two players, then no matches are saved. Still, we
-- need to persist the info that 1 player has joined the tournament
CREATE TABLE players
(
    id            uuid NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    tournament_id uuid NOT NULL REFERENCES tournaments (id),
    user_id       uuid REFERENCES users (id),
    guest_id      uuid REFERENCES guests (id)
        -- user_id XOR guest_id
        CONSTRAINT either_user_or_guest
            CHECK ( (user_id IS NOT NULL) <> (guest_id IS NOT NULL) )
);
