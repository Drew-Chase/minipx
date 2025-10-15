-- Add support for runtime detection and startup commands

-- Table for storing detected system runtimes
CREATE TABLE IF NOT EXISTS runtimes (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    version TEXT NOT NULL,
    executable_path TEXT NOT NULL,
    runtime_type TEXT NOT NULL, -- 'java', 'dotnet', 'nodejs', 'python', 'binary'
    detected_at TEXT NOT NULL,
    is_available INTEGER NOT NULL DEFAULT 1
);

-- Add startup command fields to servers table
ALTER TABLE servers ADD COLUMN startup_command TEXT;
ALTER TABLE servers ADD COLUMN runtime_id TEXT;
ALTER TABLE servers ADD COLUMN main_executable TEXT;

CREATE INDEX IF NOT EXISTS idx_runtimes_type ON runtimes(runtime_type);
CREATE INDEX IF NOT EXISTS idx_runtimes_available ON runtimes(is_available);
