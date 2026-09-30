CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY);
CREATE TABLE run_id_sequence (singleton INTEGER PRIMARY KEY CHECK (singleton = 1), value INTEGER NOT NULL CHECK (value >= 0));
INSERT INTO run_id_sequence VALUES (1, 0);
CREATE TABLE runs (id TEXT PRIMARY KEY NOT NULL, snapshot TEXT NOT NULL CHECK (json_valid(snapshot)));
CREATE TABLE run_events (
    run_id TEXT NOT NULL REFERENCES runs(id),
    position INTEGER NOT NULL CHECK (position >= 0),
    event TEXT NOT NULL CHECK (json_valid(event)),
    PRIMARY KEY (run_id, position)
);
CREATE TRIGGER immutable_event_update BEFORE UPDATE ON run_events BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;
CREATE TRIGGER immutable_event_delete BEFORE DELETE ON run_events BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;
INSERT INTO schema_migrations VALUES (1);
