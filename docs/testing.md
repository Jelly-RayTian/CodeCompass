# Testing

## Strategy

CodeCompass tests are split into two layers:

1. **Frontend tests** (Vitest + React Testing Library) — verify UI rendering,
   navigation, and state transitions.
2. **Rust tests** (`cargo test`) — verify database operations, migrations, and
   core logic.

## Frontend Tests

### Configuration

- **Runner:** Vitest with `jsdom` environment
- **Setup:** `src/test/setup.ts` — imports `@testing-library/jest-dom`, mocks
  `@tauri-apps/api/core` `invoke`, provides `mockTauriCommand` helper
- **Location:** `src/test/*.test.tsx`

### Tauri Mock

The Tauri `invoke` function is mocked globally in `src/test/setup.ts`. Each
test registers mock handlers via `mockTauriCommand(command, handler)`. Mocks
are cleared after each test (`afterEach`).

```typescript
import { mockTauriCommand } from '@/test/setup';

beforeEach(() => {
  mockTauriCommand('get_application_info', async () => ({
    name: 'CodeCompass',
    version: '0.1.0',
    buildTimestamp: '1735689600',
  }));
  mockTauriCommand('list_workspaces', async () => []);
});
```

### Running

```bash
npm run test          # run once
npm run test:watch    # watch mode
```

### Current Tests

| Test                                                           | What it verifies          |
| -------------------------------------------------------------- | ------------------------- |
| `renders the application shell with brand text`                | App mounts, brand visible |
| `shows the home page with application version on initial load` | Home page loads data      |
| `shows database status on the home page`                       | Database status renders   |
| `navigates to the Workspaces page and shows empty state`       | Nav + empty state         |
| `navigates to the Settings page`                               | Nav to settings           |

## Rust Tests

### Configuration

- **Location:** `#[cfg(test)] mod tests` inside each module
- **Isolation:** `tempfile::tempdir()` creates a unique temp directory for each
  database test. The directory is automatically cleaned up when the temp dir
  goes out of scope.

### Running

```bash
cd src-tauri
cargo test
cargo test -- --nocapture   # show println! output
```

### Current Tests

| Module                   | Test                                        | What it verifies                    |
| ------------------------ | ------------------------------------------- | ----------------------------------- |
| `db/mod.rs`              | `migration_creates_all_tables`              | V1 migration creates all 4 tables   |
| `db/connection.rs`       | `open_runs_migrations`                      | `Database::open` applies migrations |
| `db/connection.rs`       | `open_in_memory_runs_migrations`            | In-memory DB also migrates          |
| `db/connection.rs`       | `path_returns_provided_path`                | Path is stored correctly            |
| `commands/workspaces.rs` | `fetch_workspaces_returns_empty_for_new_db` | Empty list for new DB               |

## CI

GitHub Actions runs all checks on every push and PR. See
[.github/workflows/ci.yml](../.github/workflows/ci.yml).
