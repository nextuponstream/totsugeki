describe('permissions of other users', () => {
  let weeklyName = `weekly-name-${Date.now()}`
  let url: string | undefined = undefined
  it('as registered user, I can create bracket', () => {
    cy.testUserLogin()
    cy.visit('/')

    cy.get('[name=tournament]').type(weeklyName)
    cy.get('[data-test-id=next-form]').click()

    cy.get('[name=name]').type('p1{enter}')
    cy.get('[name=name]').type('p2{enter}')
    cy.get('[name=name]').type('p3{enter}')

    cy.intercept('POST', '/api/tournaments').as('createTournament')

    cy.get('[data-test-id=start-tournament]').click()

    cy.wait('@createTournament').then((interception) => {
      assert.equal(interception.response?.statusCode, 201)
    })

    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3').then(() => {
      cy.url()
        .should('contain', '/tournaments/')
        .then((v) => {
          url = v
        })
    })
  })
  it('when another user logs in, they cannot edit the bracket', () => {
    cy.testOtherUserLogin()
    cy.visit(url!)

    cy.get(`[data-test-id=winner-2-3]`).should('have.attr', 'disabled')
  })
})
