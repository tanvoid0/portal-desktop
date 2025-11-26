# Unified Invoke Architecture

## Overview

The application uses a **hybrid approach** that provides:
- **Performance**: Direct Tauri `invoke()` in desktop app (no serialization overhead)
- **Consistency**: HTTP API for browser/remote access (unified interface)
- **Flexibility**: Support for both command names and HTTP URLs

## Architecture

### Performance Comparison

| Method | Latency | Serialization | Use Case |
|--------|---------|---------------|----------|
| **Tauri `invoke()`** | ~0.1-1ms | None (direct FFI) | Desktop app (fastest) |
| **HTTP (localhost)** | ~1-5ms | JSON | Browser localhost (consistent) |
| **HTTP (remote)** | ~10-50ms | JSON + network | Remote browser (secure) |

### When to Use Each

1. **Tauri Commands** (e.g., `'get_tasks'`):
   - Use in desktop app for maximum performance
   - Automatically falls back to HTTP in browser

2. **HTTP URLs** (e.g., `'/api/tasks'` or `'https://api.example.com/tasks'`):
   - Use when you want explicit HTTP control
   - Useful for external APIs or when you need HTTP-specific features (caching, headers, etc.)

## Usage Examples

### Basic Usage

```typescript
import { unifiedInvoke } from '$lib/utils/invoke';

// Tauri command (auto-detects environment)
const tasks = await unifiedInvoke('get_tasks', { filters: null });

// HTTP URL (always uses HTTP)
const tasks = await unifiedInvoke('/api/tasks', { filters: null });
```

### With Options

```typescript
// GET request with query params
const tasks = await unifiedInvoke('/api/tasks', { status: 'active' }, {
  method: 'GET'
});

// POST with custom headers
const result = await unifiedInvoke('/api/tasks', { title: 'New Task' }, {
  method: 'POST',
  headers: {
    'X-Custom-Header': 'value'
  }
});

// Disable auth for public endpoints
const publicData = await unifiedInvoke('/api/public', null, {
  requireAuth: false
});
```

### Migration Path

**Before:**
```typescript
// Direct Tauri invoke (only works in desktop)
const tasks = await invoke('get_tasks', { filters: null });
```

**After:**
```typescript
// Unified invoke (works everywhere)
import { unifiedInvoke } from '$lib/utils/invoke';
const tasks = await unifiedInvoke('get_tasks', { filters: null });
```

## Implementation Details

### How It Works

1. **Command Detection**: Checks if input is a URL or command name
2. **Environment Detection**: Determines if running in Tauri or browser
3. **Routing**:
   - **Tauri + Command**: Direct `invoke()` (fastest)
   - **Browser + Command**: HTTP via `/api/tauri/{command}`
   - **Browser + URL**: Direct HTTP fetch
4. **Authentication**: Automatically adds auth headers for remote access

### Backward Compatibility

The `safeInvoke` function is now an alias for `unifiedInvoke`, so existing code continues to work:

```typescript
// This still works
import { safeInvoke } from '$lib/utils/invoke';
const tasks = await safeInvoke('get_tasks', { filters: null });
```

## Benefits

### 1. Performance
- **Desktop app**: Zero overhead, direct FFI calls
- **Browser**: Minimal overhead, standard HTTP

### 2. Consistency
- Same API for all environments
- Predictable behavior across platforms

### 3. Flexibility
- Can use commands or URLs interchangeably
- Easy to migrate to full HTTP if needed
- Supports external APIs seamlessly

### 4. Developer Experience
- Single import for all backend calls
- Type-safe with TypeScript
- Clear error messages

## Future Enhancements

### HTTP Server in Tauri (Optional)

For even better browser support, you could add an HTTP server in Tauri:

```rust
// src-tauri/src/http_server.rs
use axum::{Router, routing::post, Json};
use tauri::Manager;

pub fn create_router(app: tauri::AppHandle) -> Router {
    Router::new()
        .route("/api/commands/:command", post(handle_command))
        // ... more routes
}

async fn handle_command(
    Path(command): Path<String>,
    Json(payload): Json<serde_json::Value>,
    State(app): State<tauri::AppHandle>
) -> Result<Json<serde_json::Value>, String> {
    // Call Tauri command and return result
    // ...
}
```

This would allow:
- Direct HTTP access from browser (no SvelteKit proxy needed)
- Better performance for browser access
- Standard REST API interface

## Recommendations

1. **Use `unifiedInvoke` for all new code**
2. **Migrate existing `invoke()` calls gradually**
3. **Use HTTP URLs when you need HTTP-specific features**
4. **Keep Tauri commands for desktop-only features**

## Performance Notes

- **Tauri invoke**: ~100x faster than HTTP (no serialization)
- **HTTP localhost**: ~5x slower than Tauri (JSON serialization)
- **HTTP remote**: ~50x slower than Tauri (network + serialization)

For most use cases, the performance difference is negligible (< 10ms), but for high-frequency operations, Tauri invoke is preferred.

