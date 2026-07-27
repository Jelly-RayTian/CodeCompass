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

### Current coverage

The frontend suite currently contains 13 tests covering the application shell,
Home data and database-path redaction, navigation, workspace empty/error states,
retry behavior, Settings, Insights, lazy Viewer loading, and both supported
languages. The exact breakdown is maintained in
[test-matrix.md](test-matrix.md).

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

### Current coverage

The Rust suite currently contains 118 tests: 99 unit tests, 10 failure-path
tests, and 9 fixture-project integration tests. They cover migrations V1–V9,
database operations, scanner lifecycle and reconciliation, AST analysis,
resolution, graphs, symbols, references, health/evolution calculations,
task cancellation, error payloads, and end-to-end persistence. See
[test-matrix.md](test-matrix.md) for the maintained inventory.

## CI

GitHub Actions runs all checks on every push and PR. See
[.github/workflows/ci.yml](../.github/workflows/ci.yml).

Tagged releases also build both Windows installer formats, perform an
install-launch-uninstall NSIS smoke test on the clean Windows runner, and
publish installer checksums.
