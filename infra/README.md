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
