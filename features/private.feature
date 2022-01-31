
Feature: Private API feature

  Scenario: If we request open orders from a private API, it returns correct list
    Given I have some properties concerning a private API
    When I request all open orders
    Then the open orders list is presented to me
