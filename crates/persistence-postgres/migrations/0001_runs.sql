CREATE SEQUENCE run_id_seq;

CREATE TABLE runs (
    id TEXT PRIMARY KEY,
    snapshot JSONB NOT NULL CHECK (jsonb_typeof(snapshot) = 'object')
);

CREATE TABLE run_events (
    run_id TEXT NOT NULL REFERENCES runs(id),
    position INTEGER NOT NULL CHECK (position >= 0),
    event JSONB NOT NULL CHECK (jsonb_typeof(event) = 'object'),
    PRIMARY KEY (run_id, position)
);

CREATE FUNCTION reject_event_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'run events are immutable';
END;
$$;

CREATE TRIGGER run_events_immutable
BEFORE UPDATE OR DELETE ON run_events
FOR EACH ROW EXECUTE FUNCTION reject_event_mutation();
