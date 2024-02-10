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
  cy.contains('Register / Login')
})
