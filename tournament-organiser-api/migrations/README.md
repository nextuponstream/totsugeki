# Database design

## Normalization

Just normalize your entities. Don't store a JSON column if you can.

If you really need to go fast and forego normalization, just serialize your
data to json and pray you won't need to deal with the mess later.

## Primary

Always use UUIDs for consistency.

There is the users table: maybe you don't want you don't want to leak the total
count. While it does not make much sense performance-wise to make the primary
key of a type table of UUID type when you can have 1,2,3..., then you need to
worry about "is the PK 1,2,3 or UUID"?

## Types and formats

If you only have two types for a given entity, add a database CHECK constraint.

## Storing integers

Add a constraint when it's a positive integer. Default to smallint types. Stop
wasting time deciding if it's numeric(4, 0) or numeric(5, 0). Also, numeric
forces you to use dependency bigdecimal in this crate.