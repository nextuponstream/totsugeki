# Tests

The following test folder contains only integration tests which should not
be exposed through public interfaces. This mainly concerns queries and
database schema concerns like:

* is my database schema correctly deleting in cascade when I use my
  repositories (visibility: pub(crate))

This test folder directly goes against the recommended way of organizing
tests as described in the rust book (
see [link](https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests)).
However, putting those database integration tests in the `tests` folder would
force some struct using the database to expose a public API, which should not be
a public detail. This would also force the maintainer to make all tests be
end-to-end tests and go through the HTTP layer before interacting with the
database.

While this breaks the idea that `src` folder only contains unit tests, this
still allows the maintainer to build confidence by adding database tests.
There is value in making sure foreign keys, referential action (cascade on
delete) and database constraints are correctly set up.

All tests in this folder SHOULD use `sqlx::test` macro, as there are
currently no other integration which we want to test internally.
