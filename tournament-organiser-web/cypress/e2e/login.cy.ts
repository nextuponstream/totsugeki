it('shows warning when user is unknown', () => {
  cy.visit('/')

  cy.get('[data-test-id=modal]').should('not.be.visible')
  cy.contains('Register').click()
  cy.get('[data-test-id=modal]').should('be.visible')
  cy.contains('Email')
  cy.contains('Password')

  cy.intercept('POST', '/api/login').as('login')

  cy.get('[name=login]').within(() => {
    cy.get('[name=email]').type('unknown@user.ch')
    cy.get('[name=password]').type('securePass123#')
    cy.get('button').click()
  })

  cy.wait('@login').then((interception) => {
    assert.equal(interception.response?.statusCode, 404)
  })
  cy.contains('Unknown email')
})
