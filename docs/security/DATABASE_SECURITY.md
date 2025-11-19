# Database Security Configuration

## Overview

The Portal Desktop application uses SQLite for local data storage with security best practices.

## Database Location

**Location**: User's app data directory (OS-specific, user-specific)
- **Linux**: `~/.local/share/portal-desktop/portal_desktop.db`
- **macOS**: `~/Library/Application Support/portal-desktop/portal_desktop.db`
- **Windows**: `%APPDATA%\portal-desktop\portal_desktop.db`

**Security Properties**:
- ✅ User-specific (not shared between users)
- ✅ Outside project directory (never committed to git)
- ✅ Secure file permissions (0o600 on Unix, owner-only on Windows)
- ✅ Secure directory permissions (0o700 on Unix)

## Security Features

### 1. File Permissions
- **Database file**: `0o600` (read/write for owner only)
- **Database directory**: `0o700` (access for owner only)
- Permissions are set automatically on creation and after migrations

### 2. Git Exclusion
- Database files are excluded from git via `.gitignore`:
  - `*.db`
  - `*.db-shm` (SQLite shared memory)
  - `*.db-wal` (SQLite write-ahead log)
  - `*.sqlite`
  - `*.sqlite3`
  - `data/` directory

### 3. Automatic Initialization
- Database is created automatically on first run
- Migrations run automatically on startup
- Migrations are idempotent (safe to run multiple times)

### 4. Data Encryption
- **Credentials**: Encrypted at rest using AES-256-GCM
- **Database file**: SQLite file itself is not encrypted
- **Future enhancement**: Consider SQLCipher for full database encryption

## Migration Security

- Migrations run automatically on startup
- SeaORM tracks which migrations have been applied
- Failed migrations are logged and prevent app startup
- Migrations are idempotent (can be run multiple times safely)

## Best Practices

1. **Never commit database files** - Already excluded via `.gitignore`
2. **User-specific storage** - Each user has their own database
3. **Secure permissions** - Files are only accessible by the owner
4. **Automatic migration** - Schema updates happen automatically
5. **Credential encryption** - Sensitive data is encrypted before storage

## Verification

To verify database security:

```bash
# Check database location (should be in user's app data directory)
# Check file permissions (should be 600 on Unix)
ls -la ~/.local/share/portal-desktop/portal_desktop.db

# Verify database is not in git
git status | grep -i "\.db"
# Should return nothing
```

## Future Enhancements

1. **SQLCipher Integration**: Encrypt entire database file
2. **Backup Encryption**: Encrypt database backups
3. **Audit Logging**: Track database access
4. **Connection Pooling**: Limit concurrent connections
5. **Database Integrity Checks**: Periodic verification

