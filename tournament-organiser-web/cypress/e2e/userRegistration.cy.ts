it('dismiss modal by clicking outside', () => {
    cy.visit('/')

    cy.get('[data-test-id=modal]').should('not.be.visible')

    cy.contains('Register').click()

    cy.get('[data-test-id=modal]').should('be.visible')

    cy.get('[data-test-id=blurred-background-outside-modal]').click("topLeft", { force: true })

    cy.get('[data-test-id=modal]').should('not.be.visible')
})

it('visits the app root url', () => {
    cy.visit('/')
    cy.contains('Register').click()

    cy.contains('Username')
    cy.contains('Email')
    cy.contains('Password')
    cy.contains('Confirm password')

    cy.intercept('POST', '/register').as('registration')

    cy.get('form').within(() => {
        cy.get('input > [type=email]').type('jean@bon.ch')
        cy.get('input > [type=password]').first().type('someSecurePassword1234#')
        cy.get('input > [type=password]').eq(1).type('someSecurePassword1234#')
        cy.submit()
    })

    cy.wait('@registration').its('response.statusCode').should('eq', 200)

    cy.url().should('not.contain', '/register')
})
