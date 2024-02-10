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
