-- Add email field to users table for SQLite

-- SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS
-- We need to check if column exists first

-- Create a new table with all columns
CREATE TABLE IF NOT EXISTS users_new (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    is_active INTEGER NOT NULL DEFAULT 1,
    email_verified INTEGER NOT NULL DEFAULT 0,
    email_verification_token TEXT,
    email_verification_expires INTEGER,
    password_reset_token TEXT,
    password_reset_expires INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_login INTEGER
);

-- Copy data from old table (if it exists and doesn't have email column)
INSERT OR IGNORE INTO users_new (id, username, password_hash, role, is_active, created_at, updated_at, last_login)
SELECT id, username, password_hash, role, is_active, created_at, updated_at, last_login
FROM users;

-- Drop old table and rename new one
DROP TABLE IF EXISTS users;
ALTER TABLE users_new RENAME TO users;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_email_verification_token ON users(email_verification_token);
CREATE INDEX IF NOT EXISTS idx_users_password_reset_token ON users(password_reset_token);

-- Recreate trigger
CREATE TRIGGER IF NOT EXISTS update_users_updated_at 
AFTER UPDATE ON users
BEGIN
    UPDATE users SET updated_at = strftime('%s', 'now') WHERE id = NEW.id;
END;

-- Update existing admin user with email
UPDATE users SET email = 'admin@hal9.ai' WHERE username = 'admin' AND email IS NULL;