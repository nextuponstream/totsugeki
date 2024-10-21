it('allows new registered user to be deleted', () => {
  let email = `user-to-delete-${Date.now()}@domain.ch`
  let username = `user-to-delete`
  let password = 'securePassword@21321'
  cy.register(email, username, password)

  cy.login(email, password)

  cy.visit('/user/dashboard')

  cy.contains('Delete my account').click()
  cy.get('[name=deleteEmail]').type(email)
  cy.get('button').contains('Delete account').click()

  // cannot log in again
  cy.get('[data-test-id=navbar]').within(() => {
    cy.contains('Register').click()
  })
  cy.get('[name=login]').within(() => {
    cy.get('[name=email]').type(email)
    cy.get('[name=password]').type(password)

    cy.intercept('POST', '/api/login').as('login')
    cy.contains('Submit').click()
    cy.wait('@login').then((interception) => {
      assert.isNotNull(interception.response, 'response')
      assert.equal(interception.response?.statusCode, 404)
    })
    cy.contains('Unknown email')
  })
})
