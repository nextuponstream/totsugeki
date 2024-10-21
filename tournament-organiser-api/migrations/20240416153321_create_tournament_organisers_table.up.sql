-- Add up migration script here
-- Every user that creates a bracket is the de-facto organiser. Therefore, we
-- need to save that information.
-- There could be multiple organisers for a bracket (think fallback person in
-- case the first person is missing). One person can handle multiple brackets.
-- Here, we need a many-to-many relationship.
--
-- For the naming, this could have been users_brackets but players usually
-- refer to TOs. Then it feels natural to name it that way.
--
-- As of right now, if someone is part of responsible for a bracket, then only
-- one entry should be saved in this table. This table only answers the
-- question: "are you responsible for bracket X". Any extraneous info is
-- irrelevant. Creating a primary key that is not the composition of user and
-- bracket id forces us to define a unique constraint over the pair. Primary key
-- is unique by definition so we don't need to define the constraint.
--
-- I don't want to waste anymore brain time on this table because scaling to
-- infinity and beyond is a dream for the deranged. Let's not over-engineer
-- this.
CREATE TABLE tournament_organisers
(
    bracket_id uuid NOT NULL REFERENCES brackets ON DELETE CASCADE,
    user_id    uuid NOT NULL REFERENCES users ON DELETE CASCADE,
    PRIMARY KEY (bracket_id, user_id)
);
