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

    cy.get('[data-test-id=start-bracket]').click()

    cy.url().should('contain', '/3c3ebe96-c051-4d7c-bace-a8ddf5924cf8/bracket/')

    cy.contains('p1')
    cy.contains('p2')
    cy.contains('p3')
  })
  // it('the current bracket I was managing is registered in my history', () => {
  //   cy.testUserLogin()
  //   cy.visit('/')
  //   cy.get('[data-test-id=menu]').click()
  //   cy.get('[data-test-id=my-brackets]').click()
  //   cy.url().should('contain', '/3c3ebe96-c051-4d7c-bace-a8ddf5924cf8/brackets')
  // })
})

// describe('allow creating brackets without signing up', () => {
//   it("as an unregistered user, I can create bracket but with a warning that it won't be saved", () => {
//     throw new Error('implement')
//   })
//   it('if I create an account and log in', () => {
//     throw new Error('implement')
//   })
//   it('the current bracket I was managing is registered in my history', () => {
//     throw new Error('implement')
//   })
// })
