Feature: Creating brackets and find them
    
  Scenario: Someone looks for a specific bracket
    Given my-favorite-to wants to create a bracket named zurich-weekly
    When they create a bracket using discord bot
    Then they can filter results and find the created bracket
