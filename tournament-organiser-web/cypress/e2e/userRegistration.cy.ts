it('dismiss modal by clicking outside', () => {
    cy.visit('/')

    cy.get('[data-test-id=modal]').should('not.be.visible')

    cy.contains('Register').click()

    cy.get('[data-test-id=modal]').should('be.visible')

    cy.get('[data-test-id=blurred-background-outside-modal]').click("topLeft", { force: true })

    cy.get('[data-test-id=modal]').should('not.be.visible')
})

it('register user', () => {
    cy.visit('/')

    cy.get('[data-test-id=modal]').should('not.be.visible')

    cy.contains('Register').click()

    cy.get('[data-test-id=modal]').should('be.visible')

    cy.contains('Register now!').click()

    cy.url().should('contain', '/register')

    cy.contains('Email')
    cy.contains('Username')
    cy.contains('Password')
    cy.contains('Confirm password')

    cy.intercept('POST', '/register-user').as('registration')

    cy.get('[name=user-registration]').within(() => {
        cy.get('[name=username]').type('jean')
        cy.get('[name=email]').type('jean@bon.ch')
        cy.get('[name=password]').type('someSecurePassword1234#')
        cy.get('[name=confirmPassword]').type('someSecurePassword1234#')
    }).submit()

    cy.wait('@registration').its('response.statusCode').should('eq', 200)

    cy.get('[data-test-id=modal]').should('not.be.visible')
    cy.url().should('not.contain', '/register')
})
