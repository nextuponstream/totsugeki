# https://stackoverflow.com/a/38625070/29197309
psql --dbname=toa -c "copy (select datname from pg_database where datname like '%_sqlx_test%') to stdout" | while read line; do
    dropdb "$line"
done