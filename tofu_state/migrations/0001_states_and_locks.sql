-- OpenTofu state for each stack (decision 0009). The bodies are OpenTofu's
-- encrypted state files: the passphrase never reaches the Worker, so this
-- table only ever holds ciphertext.
CREATE TABLE states (
  stack TEXT NOT NULL,
  version INTEGER NOT NULL,
  body TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
  PRIMARY KEY (stack, version)
);

-- At most one lock per stack. `info` is OpenTofu's lock info JSON, returned
-- to anyone who tries to take the lock while it is held.
CREATE TABLE locks (
  stack TEXT PRIMARY KEY,
  id TEXT NOT NULL,
  info TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
