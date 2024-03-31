it('after logging out, I cannot see my profile even when I try to paste the url', () => {
  cy.testUserLogin()
  cy.visit('/')

  cy.get('[data-test-id=navbar]').within(() => {
    cy.contains('Profile').click()
  })
  cy.url().should('contain', '/user/dashboard')
  cy.get('[name=name]').should('have.value', 'test user')

  // TODO cannot assert value of disabled email input for reason? internet is
  // not helpful
  // cy.get('[name=email]').should('have.value', `test@user.ch`)

  cy.get('[data-test-id=navbar]').within(() => {
    cy.contains('Logout').click()
  })

  cy.visit('/user/dashboard')
  cy.url().should('not.contain', 'dashboard')
  cy.get('[name=name]').should('not.exist')
})
