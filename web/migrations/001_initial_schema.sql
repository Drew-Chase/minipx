-- Initial schema for minipx web dashboard

CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    domain TEXT NOT NULL UNIQUE,
    host TEXT NOT NULL DEFAULT 'localhost',
    port INTEGER NOT NULL,
    path TEXT NOT NULL DEFAULT '',
    ssl_enabled INTEGER NOT NULL DEFAULT 0,
    redirect_to_https INTEGER NOT NULL DEFAULT 0,
    listen_port INTEGER,
    status TEXT NOT NULL DEFAULT 'stopped',
    binary_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS certificates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    domain TEXT NOT NULL,
    cert_path TEXT NOT NULL,
    key_path TEXT,
    is_letsencrypt INTEGER NOT NULL DEFAULT 0,
    expiry_date TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS server_certificates (
    server_id TEXT NOT NULL,
    certificate_id TEXT NOT NULL,
    PRIMARY KEY (server_id, certificate_id),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (certificate_id) REFERENCES certificates(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS resource_metrics (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    network_in REAL NOT NULL,
    network_out REAL NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_servers_domain ON servers(domain);
CREATE INDEX IF NOT EXISTS idx_servers_status ON servers(status);
CREATE INDEX IF NOT EXISTS idx_certificates_domain ON certificates(domain);
CREATE INDEX IF NOT EXISTS idx_resource_metrics_server_timestamp ON resource_metrics(server_id, timestamp);
