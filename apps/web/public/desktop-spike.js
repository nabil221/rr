async function checkDesktopTransport() {
  const endpoint = "http://proof-api.localhost";
  const health = await fetch(`${endpoint}/api/health`);
  const healthBody = await health.json();
  if (!health.ok || healthBody.storage !== "in-memory")
    throw new Error("Health check failed");

  const created = await fetch(`${endpoint}/api/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      seed: 42,
      region: "north",
      threshold: 0,
      forceValidationFailure: false,
    }),
  });
  const run = await created.json();
  if (created.status !== 201 || run.status !== "queued")
    throw new Error("Create check failed");

  const invalid = await fetch(`${endpoint}/api/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      seed: 42,
      region: "",
      threshold: 0,
      forceValidationFailure: false,
    }),
  });
  const error = await invalid.json();
  if (
    invalid.status !== 400 ||
    error.code !== "invalid-request" ||
    error.field !== "region"
  )
    throw new Error("Typed error check failed");

  const missing = await fetch(`${endpoint}/api/runs/spike-missing-id`);
  const missingError = await missing.json();
  if (missing.status !== 404 || missingError.code !== "not-found")
    throw new Error("Missing-run error check failed");

  // This final request appears in native logs only after all body assertions pass.
  const checkpoint = await fetch(`${endpoint}/api/health`);
  if (!checkpoint.ok) throw new Error("Final transport checkpoint failed");

  document.querySelector("#results").textContent = JSON.stringify(
    {
      outcome: "PASS",
      origin: location.origin,
      health: healthBody,
      createdRun: run,
      typedError: error,
      missingRunError: missingError,
    },
    null,
    2,
  );
}

checkDesktopTransport().catch((error) => {
  document.querySelector("#results").textContent = `FAIL: ${error.message}`;
});
