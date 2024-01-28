it('dismiss modal by clicking outside', () => {
  cy.visit('/')

  cy.get('[data-test-id=modal]').should('not.be.visible')

  cy.contains('Register').click()

  cy.get('[data-test-id=modal]').should('be.visible')
  cy.get('[data-test-id=blurred-background-outside-modal]').click('topLeft', {
    force: true,
  })

  cy.get('[data-test-id=modal]').should('not.be.visible')
})

it('register user', () => {
  cy.visit('/')

  cy.get('[data-test-id=modal]').should('not.be.visible')
  cy.contains('Register').click()
  cy.get('[data-test-id=modal]').should('be.visible')
  cy.contains('Register now!').click()

  cy.url().should('contain', '/register')
  cy.contains('Email')
  cy.contains('Username')
  cy.contains('Password')
  cy.contains('Confirm password')

  cy.intercept('POST', '/api/register').as('registration')

  cy.get('[name=user-registration]').within(() => {
    cy.get('[name=name]').type('jean')
    // easy unique name ID https://stackoverflow.com/a/75183565
    // this way, no need to reset the database
    cy.get('[name=email]').type(`jean-${Date.now()}@bon.ch`)
    cy.get('[name=password]').type('someSecurePassword1234#')
    cy.get('[name=confirmPassword]').type('someSecurePassword1234#')
    cy.get('button').click()
  })

  cy.wait('@registration').then((interception) => {
    assert.isNotNull(interception.response, 'response')
    assert.equal(interception.response?.statusCode, 200)
  })

  cy.get('[data-test-id=modal]').should('not.be.visible')
  cy.url().should('not.contain', '/register')
})
