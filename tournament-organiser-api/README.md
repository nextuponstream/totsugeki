# Tournament organiser API

An api to create and manage brackets. Serves `tournament-organiser-web` if
available. 

For now, we limit the scope to tournament organiser to create brackets, add
players and update results. No persistence is in place server-side.

## Installation

### Dev setup

Chosing to not manage inside a container. Following this [guide](https://www.digitalocean.com/community/tutorials/how-to-install-postgresql-on-ubuntu-20-04-quickstart),
we install (ubuntu based distro):

```bash
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql.service

# check it works
systemctl status postgresql
```

To do postgres stuff, log in as postgres user and use the psql command

```bash
sudo -u postgres psql
```

For our dev application, let's create a postgresql user (while logged in as
postgres user)

```bash
createuser --interactive 
# Enter name of role to add: toa                     
# Shall the new role be a superuser? (y/n) y
# Shall the new role be allowed to create databases? (y/n) n
# Shall the new role be allowed to create more new roles? (y/n) n
```

To run `sqlx::test`, we need our user to have superuser privileges as discussed
[here](https://github.com/launchbadge/sqlx/discussions/2051).

Create the database associated to that user

```bash
createdb toa
```

For ident auth, create a linux user (as non-postgres user)

```bash
sudo adduser toa
```

Check you can acces toa database with

```bash
sudo -u toa psql
```

To have an easy time with the database connection string, we associate a
password to toa user

```bash
sudo -u postgres psql
#postgres=# \password toa
#Enter new password: toa
#postgres=# \q
```

Install sqlx-cli to run migration

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
# check everything is ok
#sqlx --version
```

Create a dotenv file using the example one for sqlx to use the correct database

```bash
cp .env.example .env
```

## Migration

Add new migration (-r for creating an up and down migration)

```bash
sqlx migrate add -r <new_migration>
```

Run migrations

```bash
sqlx migrate run
```

Check if it was applied

```
sudo -u toa psql
psql (14.10 (Ubuntu 14.10-0ubuntu0.22.04.1))
Type "help" for help.

toa=> \dt
             List of relations
 Schema |       Name       | Type  | Owner 
--------+------------------+-------+-------
 public | _sqlx_migrations | table | toa
 public | users            | table | toa
(2 rows)
```

## Creating a new query

After creating a new `query!`, don't forget to check into version control its
metadata for CI.

```bash
cargo sqlx prepare
```
