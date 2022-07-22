Feature: Join bracket as player
    Scenario: Many players join a bracket
        Given my-favorite-to has created a bracket named basel-weekly
        When the-new-lad, the-old-time-player and 6 other players join
        Then there is enough people for an 8 participants tournament
