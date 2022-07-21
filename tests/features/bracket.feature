Feature: Creating brackets and find them
    
  Scenario: Someone creates a bracket
    Given my-favorite-to wants to create a bracket named basel-weekly
    When the new bracket originates from discord server of organiser FancyBar
    When the organiser has internal id 1
    When they create a bracket using discord bot
    Then they search the newly created bracket with the discord bot and find it

  Scenario: Someone looks for a specific bracket
    Given my-favorite-to wants to create a bracket named zurich-weekly
    When the new bracket originates from discord server of organiser FancyBar
    When the organiser has internal id 1
    When they create a bracket using discord bot
    Then they can filter results and find the created bracket
