# Conventions

Conventions over indecision

## Sqlx

* deserialize rows (`query_as!`) into `<EntityName>Record` struct
* generate UUID application side for inserting rows
    * use structs with a field `id` of type `uuid::UUID`

### About IDs

While allowing comparison such as `tournament_id == player_id` is wrong if both
IDs are of same type, the ergonomics of being forced to implement

* struct `TournamentID` for wrapping a UUID
* deref implementation for `TournamentID` and others like Debug, Display...
* type coercion syntax (`tournament.get_id().raw()`)

is not worth it over simply using a UUID. If this was a tightly regulated
industry scenario and instead of IDs it was metrics, then yes, use wrappers,
crates and other things to secure your types.

This is not worth the cost. Use automated tests to verify the IDs are doing
what they are supposed to and don't rely on the compiler for this kind of thing.

(You can see I have to talk myself out of writing more code)
