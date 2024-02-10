describe('Homepage proposes you to create a bracket', () => {
  it('About page is reachable', () => {
    cy.visit('/about')
    cy.contains('Github')
  })
})
