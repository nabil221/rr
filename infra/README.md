# Local infrastructure

This directory contains only the local browser-server dependency: PostgreSQL.

From the repository root:

```text
docker compose -f infra/compose.yaml up -d
docker compose -f infra/compose.yaml ps
docker compose -f infra/compose.yaml logs postgres
docker compose -f infra/compose.yaml stop
docker compose -f infra/compose.yaml down
```

To deliberately reset this project’s database volume:

```text
docker compose -f infra/compose.yaml down --volumes
```

The volume name is project-specific (`local_stack_proof_postgres_data`). This reset removes only this project’s local Postgres data. Desktop mode does not use this container.

## Persistence adapter tests

With the container running, run `npm run test:postgres` at the repository root.
These explicit integration tests require the project database at `127.0.0.1:54329`.
They create unique `stack_proof_test_*` schemas and remove only those schemas;
they never reset the volume or modify `public` or the application's `stack_proof` schema.
Ordinary `npm run test` skips these database-dependent tests.

The adapter applies versioned migrations automatically on connection. Its database URL
must target exactly `127.0.0.1:54329/local_stack_proof`; remote and other databases are rejected.
The fixture's documented username/password are local development defaults, not production credentials.
