Feature: Creating brackets and find them
    
  Scenario: Someone creates a bracket
    Given my-favorite-to wants to create a bracket named basel-weekly
    When they create a bracket using discord bot
    Then they search the newly created bracket with the discord bot and find it

  Scenario: Someone looks for a specific bracket
    Given my-favorite-to wants to create a bracket named zurich-weekly
    When they create a bracket using discord bot
    Then they can filter results and find the created bracket
