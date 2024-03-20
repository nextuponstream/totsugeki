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
  let email = `someMail-${Date.now()}@gmail.com`
  let password = 'guestPass#)2'
  it("as an unregistered user, I can create bracket but with a warning that it won't be saved", () => {
    cy.guestSession(weeklyName, email)
    cy.visit('/')

    cy.get('[name=bracket]').type(weeklyName)
    cy.get('[data-test-id=next-form]').click()

    cy.get('[name=name]').type('p1{enter}')
    cy.get('[name=name]').type('p2{enter}')
    cy.get('[name=name]').type('p3{enter}')

    cy.intercept('POST', '/api/guest/brackets').as('createBracket')

    cy.get('[data-test-id=start-bracket]').click()

    cy.wait('@createBracket').then((interception) => {
      assert.equal(interception.response?.statusCode, 200)
    })

    cy.url().should('contain', '/brackets/guest')
    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3')
    cy.contains('This bracket is currently unsaved')

    cy.submitResult(2, 3, 2, 1, 'winner')
    cy.submitResult(1, 2, 0, 2, 'winner')
    cy.submitResult(2, 3, 2, 0, 'loser')
    cy.submitResult(1, 2, 0, 2, 'grand-finals')
    cy.submitResult(1, 2, 2, 0, 'grand-finals-reset')
  })
  // TODO finish story
  it('if I create an account', () => {
    cy.register(email, 'guestToUser', password)
  })
  it('after login in, the current bracket I was managing can be registered in my history', () => {
    cy.guestSession(weeklyName, email)
    cy.login(email, password)
    cy.contains(weeklyName)

    cy.intercept('POST', '/api/brackets/save').as('saveBracket')

    cy.contains('Save bracket').click()
    cy.wait('@saveBracket').then((interception) => {
      assert.equal(interception.response?.statusCode, 201)
    })

    cy.url().should('not.contain', '/brackets/guest')
  })
})
