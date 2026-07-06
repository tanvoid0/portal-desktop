# Invoke Architecture

## The standard: `invokeClient`

All backend (Tauri command) access goes through the typed client in
[`src/lib/utils/invokeClient.ts`](../src/lib/utils/invokeClient.ts). It routes
transparently between environments:

- **Desktop (Tauri)** → direct `invoke()` (no serialization overhead)
- **Browser localhost** → HTTP via the SvelteKit `/api/tauri/{command}` proxy
- **Remote browser** → authenticated HTTP

> Historical note: earlier docs described a `unifiedInvoke` helper in
> `$lib/utils/invoke`. That abstraction was never adopted (zero imports).
> `invokeClient` is the single source of truth — do not reintroduce a parallel
> wrapper.

## Layering rules

1. **Routes and components never import `@tauri-apps/api/core` or call
   `invoke()` directly.** They call a domain **service**.
2. **Services** (`src/lib/domains/<domain>/services/*Service.ts`) are the only
   place backend commands are invoked. Each command gets a typed method.
3. **Data loading** uses the TanStack Query factories under
   `src/lib/domains/shared/query/` (`createXQuery`, mutations that invalidate +
   toast). See `.cursor/rules/data-loading-conventions.md`.

## Usage

```typescript
// In a service — the ONLY place raw invoke lives.
import { invokeClient } from "$lib/utils/invokeClient";

const process = await invokeClient.request<TerminalProcess>(
  "create_terminal_process",
  { data: { request } },
);
```

```typescript
// In a route/component — go through the service.
import { sdkService } from "$lib/domains/sdk/services/sdkService";

const managers = await sdkService.detectManagers();
```

Tauri v2 maps camelCase argument keys to the command's snake_case params, so
`{ sdkType }` reaches `sdk_type`.

## Streaming

For event-stream commands use `invokeClient.live(...)`, which bridges Tauri
events and their HTTP equivalent. Errors surface through the client's error
interceptors — services should let them propagate rather than swallowing with
`console.error`.
