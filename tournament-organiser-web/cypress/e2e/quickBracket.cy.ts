describe('creating brackets as a registered user', () => {
  let weeklyName = `weekly-name-${Date.now()}`
  it('as registered user, I can create bracket', () => {
    cy.testUserLogin()
    cy.visit('/')

    cy.get('[name=bracket]').type(weeklyName)
    cy.get('[data-test-id=next-form]').click()

    cy.get('[name=name]').type('p1{enter}')
    cy.get('[name=name]').type('p2{enter}')
    cy.get('[name=name]').type('p3{enter}')

    cy.intercept('POST', '/api/brackets').as('createBracket')

    cy.get('[data-test-id=start-bracket]').click()

    cy.wait('@createBracket').then((interception) => {
      assert.equal(interception.response?.statusCode, 201)
    })

    cy.url().should('contain', '/brackets/')

    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3')
  })
  // TODO finish this part
  // it('the current bracket I was managing is registered in my history', () => {
  //   cy.testUserLogin()
  //   cy.visit('/')
  //   cy.get('[data-test-id=menu]').click()
  //   cy.get('[data-test-id=my-brackets]').click()
  //   cy.url().should('contain', '/3c3ebe96-c051-4d7c-bace-a8ddf5924cf8/brackets')
  // })
})

describe('allow creating brackets without signing up', () => {
  let weeklyName = `weekly-name-${Date.now()}`
  it("as an unregistered user, I can create bracket but with a warning that it won't be saved", () => {
    cy.visit('/')

    cy.get('[name=bracket]').type(weeklyName)
    cy.get('[data-test-id=next-form]').click()

    cy.get('[name=name]').type('p1{enter}')
    cy.get('[name=name]').type('p2{enter}')
    cy.get('[name=name]').type('p3{enter}')

    cy.intercept('POST', '/api/brackets').as('createBracket')

    cy.get('[data-test-id=start-bracket]').click()

    cy.wait('@createBracket').then((interception) => {
      assert.equal(interception.response?.statusCode, 201)
    })

    cy.url().should('contain', '/brackets/')

    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3')
    cy.contains('This bracket is currently unsaved')
  })
  it('if I create an account and log in', () => {
    throw new Error('implement')
  })
  it('the current bracket I was managing is registered in my history', () => {
    throw new Error('implement')
  })
})
