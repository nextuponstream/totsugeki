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
  cy.get('[data-test-id=navbar]').within(() => {
    cy.contains('Register / Login').click()
  })

  cy.get('[name=login]').within(() => {
    cy.get('[name=email]').type(email)
    cy.get('[name=password]').type(password)

    cy.intercept('POST', '/api/login').as('login')
    cy.contains('Submit').click()
    cy.wait('@login').then((interception) => {
      assert.isNotNull(interception.response, 'response')
      assert.equal(interception.response?.statusCode, 200)
    })
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
    cy.get('[data-test-id=navbar]').within(() => {
      cy.contains('Register').click()
    })
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

type Bracket = 'winner' | 'loser' | 'grand-finals' | 'grand-finals-reset'

Cypress.Commands.add(
  'submitResult',
  (
    firstSeed: number,
    secondSeed: number,
    scoreP1: number,
    scoreP2: number,
    bracket: Bracket
  ) => {
    cy.get(`[data-test-id=${bracket}-${firstSeed}-${secondSeed}]`).click()
    cy.contains(`${scoreP1} - ${scoreP2}`).click()

    cy.intercept('POST', '/api/report-result').as('reportFirstMatch')

    cy.get('[data-test-id=submit-match-result]').click()

    cy.wait('@reportFirstMatch').then((interception) => {
      assert.equal(interception.response?.statusCode, 200)
    })
  }
)

Cypress.Commands.add('guestSession', (weeklyName: string, email: string) => {
  cy.session(['guest', weeklyName, email], () => {
    cy.visit('/')

    cy.get('[name=bracket]').type(weeklyName)
    cy.get('[data-test-id=next-form]').click()

    cy.get('[name=name]').type('p1{enter}')
    cy.get('[name=name]').type('p2{enter}')
    cy.get('[name=name]').type('p3{enter}')

    cy.intercept('POST', '/api/guest/brackets').as('createBracket')

    cy.get('[data-test-id=start-bracket]').click()

    cy.wait('@createBracket').then((interception) => {
      assert.equal(interception.response?.statusCode, 200)
    })

    cy.url().should('contain', '/brackets/')

    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3')
    cy.contains('This bracket is currently unsaved')

    cy.submitResult(2, 3, 2, 1, 'winner')
    cy.submitResult(1, 2, 0, 2, 'winner')
    cy.submitResult(2, 3, 2, 0, 'loser')
    cy.submitResult(1, 2, 0, 2, 'grand-finals')
    cy.submitResult(1, 2, 2, 0, 'grand-finals-reset')
  })
})

Cypress.Commands.add('playerLogin', (email: string, password: string) => {
  cy.session([email], () => {
    cy.visit('/')

    cy.get('[data-test-id=modal]').should('not.be.visible')
    cy.get('[data-test-id=navbar]').within(() => {
      cy.contains('Register').click()
    })
    cy.get('[data-test-id=modal]').should('be.visible')
    cy.contains('Email')
    cy.contains('Password')

    cy.intercept('POST', '/api/login').as('login')

    cy.get('[name=login]').within(() => {
      cy.get('[name=email]').type(email)
      cy.get('[name=password]').type(password)
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
      /**
       * no sessions are saved
       * @param email
       * @param password
       */
      login(email: string, password: string): Chainable<void>

      /**
       * Session is saved
       * @param email
       * @param password
       */
      playerLogin(email: string, password: string): Chainable<void>

      register(
        email: string,
        username: string,
        password: string
      ): Chainable<void>

      testUserLogin(): Chainable<void>

      submitResult(
        firstSeed: number,
        secondSeed: number,
        scoreP1: number,
        scoreP2: number,
        bracket: Bracket
      ): Chainable<void>

      guestSession(weeklyName: string, email: string): Chainable<void>

      //   drag(subject: string, options?: Partial<TypeOptions>): Chainable<Element>
      //   dismiss(subject: string, options?: Partial<TypeOptions>): Chainable<Element>
      //   visit(originalFn: CommandOriginalFn, url: string, options: Partial<VisitOptions>): Chainable<Element>
    }
  }
}

export {}
