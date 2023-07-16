describe('Homepage proposes you to create a bracket', () => {
  it('visits the app root url', () => {
    cy.visit('/')
    cy.contains('Create a new bracket')
  })
})
