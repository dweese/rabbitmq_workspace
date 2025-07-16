# Drop and recreate the pagila database
dropdb pagila
createdb pagila

# Now load fresh
cd ~/dev/rust/rabbitmq_workspace/artifacts/pagila-master
psql -d pagila -f pagila-schema.sql
psql -d pagila -f pagila-data.sql
