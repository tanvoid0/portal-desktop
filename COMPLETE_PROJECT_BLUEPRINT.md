# Complete Project Blueprint
## Modern Desktop App with Tauri + SvelteKit + Rust

Based on lessons learned from 3 production projects: **logs-explorer**, **portal-desktop**, and **terminux**.

## 🚀 Quick Start

### 1. Project Initialization (CLI-First Approach)
```bash
# Create Tauri project with latest versions
npm create tauri-app@latest my-app
cd my-app

# Install pnpm (faster, more reliable)
npm install -g pnpm

# Install dependencies
pnpm install

# Add shadcn/ui (CRITICAL - no raw HTML components)
npx shadcn@latest init
npx shadcn@latest add button card input select modal toast
```

### 2. Essential Dependencies
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.8.0",
    "@tauri-apps/cli": "^2.8.4",
    "svelte": "^5.0.0",
    "sveltekit": "^2.0.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.0.0"
  }
}
```

## 🏗️ Architecture Principles

### 1. Domain-Driven Design (DDD)
```
src/lib/domains/
├── shared/           # Cross-cutting concerns
│   ├── services/     # Logger, Cache, EventBus
│   ├── stores/       # Global state
│   └── types/        # Shared interfaces
├── projects/         # Project management
│   ├── services/     # Business logic
│   ├── stores/       # Domain state
│   ├── components/   # UI components
│   └── types/        # Domain types
└── terminal/         # Terminal functionality
    ├── services/
    ├── stores/
    ├── components/
    └── types/
```

### 2. Import Pattern (CRITICAL)
```typescript
// ✅ CORRECT - Use domain imports
import { getProjects } from '$lib/domains/projects';
import { logger } from '$lib/domains/shared';

// ❌ WRONG - No relative imports
import { getProjects } from '../../utils/dataLoader';
```

### 3. Svelte 5 Runes (From Day One)
```svelte
<script lang="ts">
  // ✅ CORRECT - Use runes
  interface Props {
    title: string;
    onSave?: (data: any) => void;
  }
  
  const { title, onSave }: Props = $props();
  let formData = $state({});
  const isValid = $derived(formData.name?.length > 0);
  
  $effect(() => {
    if (isValid) {
      logger.info('Form is valid', { context: 'MyComponent' });
    }
  });
</script>

<!-- ❌ WRONG - Don't use old patterns -->
<script>
  export let title = '';
  $: isValid = formData.name?.length > 0;
</script>
```

## 🔧 Core Systems

### 1. Centralized Logging (NO CONSOLE.LOG)
```typescript
// src/lib/domains/shared/services/logger.ts
export interface LogContext {
  context: string;
  data?: Record<string, any>;
  traceId?: string;
}

export const logger = {
  info: (message: string, context: LogContext) => { /* structured logging */ },
  error: (message: string, context: LogContext & { error?: Error }) => { /* error logging */ },
  debug: (message: string, context: LogContext) => { /* debug logging */ }
};

// Usage
logger.info('User created project', { 
  context: 'ProjectService', 
  data: { projectId: '123' } 
});
```

### 2. Smart Backend Calls (IPC/HTTP Fallback)
```typescript
// src/utils/tauriUtils.ts
export async function smartBackendCall<T>(
  command: string,
  args?: any,
  fallback?: T
): Promise<T> {
  if (isTauriAvailable()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(command, args);
    } catch (error) {
      logger.warn(`Tauri IPC failed for ${command}`, { context: 'TauriUtils', error });
    }
  }
  
  // HTTP fallback for development
  return makeHttpApiCall<T>(convertTauriCommandToHttpEndpoint(command));
}
```

### 3. Domain Services (Singleton Pattern)
```typescript
// src/lib/domains/projects/services/projectService.ts
class ProjectService {
  private static instance: ProjectService;
  
  static getInstance(): ProjectService {
    if (!ProjectService.instance) {
      ProjectService.instance = new ProjectService();
    }
    return ProjectService.instance;
  }
  
  async getProjects(): Promise<Project[]> {
    try {
      const projects = await smartBackendCall<Project[]>('get_projects');
      logger.info('Projects loaded', { 
        context: 'ProjectService', 
        data: { count: projects.length } 
      });
      return projects;
    } catch (error) {
      logger.error('Failed to load projects', { 
        context: 'ProjectService', 
        error 
      });
      throw error;
    }
  }
}

export const projectService = ProjectService.getInstance();
```

### 4. Domain Stores (Svelte 5)
```typescript
// src/lib/domains/projects/stores/projectStore.ts
import { writable, derived } from 'svelte/store';
import { createLoadingState } from '../../shared/stores/loadingState';
import { projectService } from '../services/projectService';

export const projects = writable<Project[]>([]);
export const projectsLoading = createLoadingState();

export const isLoading = derived(projectsLoading, $loading => $loading.isLoading);
export const hasError = derived(projectsLoading, $loading => $loading.hasError);

export const projectActions = {
  async loadProjects(): Promise<void> {
    try {
      projectsLoading.setLoading(true);
      const projectsData = await projectService.getProjects();
      projects.set(projectsData);
    } catch (error) {
      projectsLoading.setError(error instanceof Error ? error.message : 'Failed to load projects');
    } finally {
      projectsLoading.setLoading(false);
    }
  }
};
```

## 🎨 UI Components (shadcn/ui First)

### 1. Component Structure
```svelte
<!-- src/lib/components/ui/Button.svelte -->
<script lang="ts">
  import { cn } from '$lib/utils/cn';
  
  interface Props {
    variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
    size?: 'default' | 'sm' | 'lg' | 'icon';
    className?: string;
    children?: () => any;
    [key: string]: any;
  }
  
  const { 
    variant = 'default', 
    size = 'default', 
    className = '', 
    children,
    ...rest 
  }: Props = $props();
</script>

<button 
  class={cn(
    'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    'disabled:pointer-events-none disabled:opacity-50',
    {
      'bg-primary text-primary-foreground hover:bg-primary/90': variant === 'default',
      'bg-destructive text-destructive-foreground hover:bg-destructive/90': variant === 'destructive',
      'border border-input bg-background hover:bg-accent': variant === 'outline',
      'bg-secondary text-secondary-foreground hover:bg-secondary/80': variant === 'secondary',
      'hover:bg-accent hover:text-accent-foreground': variant === 'ghost',
      'text-primary underline-offset-4 hover:underline': variant === 'link',
    },
    {
      'h-10 px-4 py-2': size === 'default',
      'h-9 rounded-md px-3': size === 'sm',
      'h-11 rounded-md px-8': size === 'lg',
      'h-10 w-10': size === 'icon',
    },
    className
  )}
  {...rest}
>
  {@render children?.()}
</button>
```

### 2. Event Handling (Callback Props)
```svelte
<!-- Component with callback props -->
<script lang="ts">
  interface Props {
    onSave?: (data: FormData) => void;
    onCancel?: () => void;
  }
  
  const { onSave, onCancel }: Props = $props();
  
  function handleSubmit() {
    onSave?.(formData);
  }
</script>

<!-- Parent usage -->
<MyComponent 
  onSave={(data) => logger.info('Form saved', { context: 'Parent', data })}
  onCancel={() => setModalOpen(false)}
/>
```

## 🗄️ Backend Architecture (Rust/Tauri)

### 1. Domain Structure
```
src-tauri/src/
├── lib.rs              # Main entry point
├── database.rs         # Central database manager
├── commands.rs         # Central commands registry
├── projects/           # Projects domain
│   ├── entities/       # SeaORM entities
│   ├── services/       # Business logic
│   ├── repositories/   # Data access
│   └── commands.rs     # Tauri commands
└── shared/             # Shared utilities
    ├── logging.rs      # Structured logging
    └── types.rs        # Shared types
```

### 2. SeaORM Integration
```rust
// src-tauri/src/database.rs
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use sea_orm_migration::prelude::*;

pub struct DatabaseManager {
    pub connection: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, DbErr> {
        let database_url = "sqlite://./app.db";
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(8))
            .idle_timeout(std::time::Duration::from_secs(8));

        let connection = Database::connect(opt).await?;
        
        // Run migrations
        Migrator::up(&connection, None).await?;
        
        Ok(Self { connection })
    }
}
```

### 3. Tauri Commands
```rust
// src-tauri/src/projects/commands.rs
use tauri::State;
use crate::database::DatabaseManager;
use crate::projects::services::ProjectService;

#[tauri::command]
pub async fn get_projects(
    db: State<'_, DatabaseManager>
) -> Result<Vec<Project>, String> {
    let service = ProjectService::new(db.connection.clone());
    service.get_projects().await
        .map_err(|e| e.to_string())
}

// Register in main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_projects,
            create_project,
            update_project,
            delete_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 📁 File Structure Template

```
my-app/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   └── ui/              # shadcn/ui components
│   │   ├── domains/
│   │   │   ├── shared/          # Cross-cutting concerns
│   │   │   │   ├── services/    # Logger, Cache, EventBus
│   │   │   │   ├── stores/      # Global state
│   │   │   │   └── types/       # Shared interfaces
│   │   │   ├── projects/        # Project domain
│   │   │   │   ├── services/    # Business logic
│   │   │   │   ├── stores/      # Domain state
│   │   │   │   ├── components/  # UI components
│   │   │   │   └── types/       # Domain types
│   │   │   └── terminal/        # Terminal domain
│   │   │       ├── services/
│   │   │       ├── stores/
│   │   │       ├── components/
│   │   │       └── types/
│   │   └── utils/               # Shared utilities
│   ├── routes/                  # SvelteKit routes
│   └── app.html
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── database.rs
│   │   ├── commands.rs
│   │   ├── projects/            # Projects domain
│   │   │   ├── entities/
│   │   │   ├── services/
│   │   │   ├── repositories/
│   │   │   └── commands.rs
│   │   └── shared/              # Shared utilities
│   │       ├── logging.rs
│   │       └── types.rs
│   └── Cargo.toml
├── package.json
├── tailwind.config.js
├── tsconfig.json
└── vite.config.js
```

## ⚙️ Configuration Files

### 1. package.json
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "eslint .",
    "format": "prettier --write .",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

### 2. tailwind.config.js
```javascript
import { fontFamily } from "tailwindcss/defaultTheme";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", ...fontFamily.sans],
        mono: ["JetBrains Mono", ...fontFamily.mono],
      },
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
    },
  },
  plugins: [],
};
```

### 3. tsconfig.json
```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler",
    "target": "ES2022",
    "module": "ESNext",
    "lib": ["ES2022", "DOM", "DOM.Iterable"]
  }
}
```

## 🚫 Critical Anti-Patterns (Avoid These)

### 1. Console Usage
```typescript
// ❌ NEVER DO THIS
console.log('Debug message');
console.error('Error occurred');

// ✅ ALWAYS DO THIS
logger.info('Debug message', { context: 'MyComponent' });
logger.error('Error occurred', { context: 'MyComponent', error });
```

### 2. Raw HTML Components
```svelte
<!-- ❌ NEVER DO THIS -->
<button onclick={handleClick}>Click me</button>
<input type="text" bind:value={inputValue} />

<!-- ✅ ALWAYS DO THIS -->
<Button onclick={handleClick}>Click me</Button>
<Input type="text" bind:value={inputValue} />
```

### 3. Relative Imports
```typescript
// ❌ NEVER DO THIS
import { getProjects } from '../../utils/dataLoader';
import { logger } from '../../../shared/logger';

// ✅ ALWAYS DO THIS
import { getProjects } from '$lib/domains/projects';
import { logger } from '$lib/domains/shared';
```

### 4. Old Svelte Patterns
```svelte
<!-- ❌ NEVER DO THIS -->
<script>
  export let title = '';
  $: isValid = formData.name?.length > 0;
</script>

<!-- ✅ ALWAYS DO THIS -->
<script lang="ts">
  interface Props {
    title: string;
  }
  const { title }: Props = $props();
  const isValid = $derived(formData.name?.length > 0);
</script>
```

## 🎯 Development Workflow

### 1. Start Development
```bash
# Start Tauri development (includes both frontend and backend)
pnpm run tauri:dev

# Use the desktop app window, NOT the browser
# Browser will show CORS errors - this is expected
```

### 2. Code Quality Checks
```bash
# Type checking
pnpm run check

# Linting
pnpm run lint

# Formatting
pnpm run format

# Build verification
pnpm run build
```

### 3. Testing Strategy
```bash
# Unit tests
pnpm run test

# Component tests
pnpm run test:ui

# Coverage
pnpm run test:coverage
```

## 📋 Pre-Production Checklist

- [ ] All `console.log` statements replaced with logger
- [ ] All raw HTML components replaced with shadcn/ui
- [ ] All relative imports converted to domain imports
- [ ] All Svelte 5 runes implemented (no `export let` or `$:`)
- [ ] All event handlers use callback props (no `createEventDispatcher`)
- [ ] All TypeScript strict mode compliance
- [ ] All accessibility requirements met (WCAG 2.1 AA)
- [ ] All performance targets achieved
- [ ] All security validations implemented
- [ ] All cross-platform testing completed

## 🚀 Deployment

### 1. Build for Production
```bash
# Build Tauri app
pnpm run tauri:build

# Output will be in src-tauri/target/release/
```

### 2. Distribution
- **Linux**: AppImage and .deb packages
- **macOS**: .dmg packages (Intel and Apple Silicon)
- **Windows**: .msi and .exe installers

## 📚 Key Lessons Learned

1. **CLI-First**: Always use official CLI tools for latest versions
2. **shadcn/ui Foundation**: Build on shadcn/ui, never raw HTML
3. **Domain-Driven Design**: Organize by business domains, not technical layers
4. **Structured Logging**: No console.log from day one
5. **Svelte 5 Runes**: Use modern patterns from the start
6. **Type Safety**: TypeScript strict mode everywhere
7. **Error Handling**: Comprehensive error boundaries and recovery
8. **Performance**: Optimize for < 2s startup, < 300MB memory
9. **Accessibility**: WCAG 2.1 AA compliance from the start
10. **Testing**: Comprehensive test coverage before production

This blueprint ensures a production-ready, maintainable, and scalable desktop application with modern best practices.
