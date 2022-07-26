Feature: Generating organiser when creating bracket

    Scenario: Organiser run another bracket
        Given my-favorite-to wants to create a bracket named basel-weekly-return
        When the organiser FancyBar has already ran a bracket named basel-weekly
        When they create a bracket using discord bot
        Then there is only one organiser with two brackets named basel-weekly and basel-weekly-return
