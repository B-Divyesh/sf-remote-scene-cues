CREATE TABLE IF NOT EXISTS rooms (
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  host_token_hash TEXT NOT NULL,
  join_secret_hash TEXT NOT NULL,
  current_index INTEGER NOT NULL DEFAULT -1,
  webhook_url TEXT,
  webhook_secret TEXT,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cues (
  id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  scene TEXT NOT NULL,
  name TEXT NOT NULL,
  UNIQUE(room_id, position)
);

CREATE TABLE IF NOT EXISTS controllers (
  id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'pending',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cue_events (
  id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  sequence INTEGER NOT NULL,
  cue_id TEXT NOT NULL,
  scene TEXT NOT NULL,
  cue_name TEXT NOT NULL,
  controller_name TEXT NOT NULL,
  webhook_status TEXT NOT NULL DEFAULT 'not_configured',
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cues_room ON cues(room_id, position);
CREATE INDEX IF NOT EXISTS idx_controllers_room ON controllers(room_id, created_at);
CREATE INDEX IF NOT EXISTS idx_events_room ON cue_events(room_id, created_at DESC);

