
Feature: Public API feature

  Scenario: If I request sever time from public api, it returns correct format
    Given I have link to a public api endpoint returning server time
    When I request server time
    Then the server time format is correct

  Scenario: If I request info about asset pair from public api, it returns correct data
    Given I have link to a public api endpoint returning asset pair info
    When I request asset pair info
    Then the asset pair info format is correct
