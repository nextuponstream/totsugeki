describe('Homepage proposes you to create a bracket', () => {
    it('About page is reachable', () => {
      cy.visit('/about')
      cy.get('[data-test-id=about]').click()
      cy.contains('Github')
    })
  })