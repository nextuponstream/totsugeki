#!/usr/bin/env bash

# kill jetbrains connection (datasource tool) and every other
# https://stackoverflow.com/a/44829330
psql --dbname=toa -c 'SELECT
pg_terminate_backend(pg_stat_activity.pid) FROM
pg_stat_activity WHERE datname = current_database() AND
pid <> pg_backend_pid();'