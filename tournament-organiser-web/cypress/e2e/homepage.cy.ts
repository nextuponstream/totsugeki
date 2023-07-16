describe('Homepage proposes you to create a bracket', () => {
  it('visits the app root url', () => {
    cy.visit('/')
    cy.contains('Create a new bracket')
  })
  it('About page is reachable', () => {
    cy.visit('/')
    cy.get('[data-test-id=about]').click()
    cy.url().should('contain', '/about')
  })
})
