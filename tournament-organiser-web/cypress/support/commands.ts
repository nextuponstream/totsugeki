/// <reference types="cypress" />
// ***********************************************
// This example commands.ts shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
//
//
// -- This is a parent command --
Cypress.Commands.add('login', (email: string, password: string) => {
  cy.visit('/')
  cy.contains('Register / Login').click()
  cy.get('[name=email]').type(email)
  cy.get('[name=password]').type(password)

  cy.intercept('POST', '/api/login').as('login')
  cy.contains('Submit').click()
  cy.wait('@login').then((interception) => {
    assert.isNotNull(interception.response, 'response')
    assert.equal(interception.response?.statusCode, 200)
    // TODO body contains user_id
  })
})
Cypress.Commands.add(
  'register',
  (email: string, username: string, password: string) => {
    cy.visit('/register')
    cy.contains('Email')
    cy.contains('Username')
    cy.contains('Password')
    cy.contains('Confirm password')

    cy.intercept('POST', '/api/register').as('registration')

    cy.get('[name=user-registration]').within(() => {
      cy.get('[name=name]').type(username)
      cy.get('[name=email]').type(email)
      cy.get('[name=password]').type(password)
      cy.get('[name=confirmPassword]').type(password)
      cy.get('button').click()
    })

    cy.wait('@registration').then((interception) => {
      assert.isNotNull(interception.response, 'response')
      assert.equal(interception.response?.statusCode, 200)
    })
    cy.url().should('not.contain', 'register')
  }
)

Cypress.Commands.add('testUserLogin', () => {
  cy.session(['test@user.ch'], () => {
    cy.visit('/')

    cy.get('[data-test-id=modal]').should('not.be.visible')
    cy.contains('Register').click()
    cy.get('[data-test-id=modal]').should('be.visible')
    cy.contains('Email')
    cy.contains('Password')

    cy.intercept('POST', '/api/login').as('login')

    cy.get('[name=login]').within(() => {
      cy.get('[name=email]').type('test@user.ch')
      cy.get('[name=password]').type('securePass123#')
      cy.get('button').click()
    })

    cy.wait('@login').then((interception) => {
      assert.isNotNull(interception.response, 'response')
      assert.equal(interception.response?.statusCode, 200)
    })
  })
})
//
//
// -- This is a child command --
// Cypress.Commands.add('drag', { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add('dismiss', { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This will overwrite an existing command --
// Cypress.Commands.overwrite('visit', (originalFn, url, options) => { ... })
//
declare global {
  namespace Cypress {
    interface Chainable {
      login(email: string, password: string): Chainable<void>
      register(
        email: string,
        username: string,
        password: string
      ): Chainable<void>
      testUserLogin(): Chainable<void>

      //   drag(subject: string, options?: Partial<TypeOptions>): Chainable<Element>
      //   dismiss(subject: string, options?: Partial<TypeOptions>): Chainable<Element>
      //   visit(originalFn: CommandOriginalFn, url: string, options: Partial<VisitOptions>): Chainable<Element>
    }
  }
}

export {}
