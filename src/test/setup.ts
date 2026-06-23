import { afterEach, vi } from 'vitest';
import '@testing-library/jest-dom';

/**
 * A registry of mock handlers for Tauri commands, created via `vi.hoisted`
 * so it is available inside the hoisted `vi.mock` factory below.
 */
const { handlers } = vi.hoisted(() => ({
  handlers: new Map<string, (args?: Record<string, unknown>) => unknown>(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string, args?: Record<string, unknown>) => {
    const handler = handlers.get(command);
    if (!handler) {
      throw new Error(
        `No mock handler registered for Tauri command: ${command}`,
      );
    }
    return handler(args);
  }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => undefined),
}));

// Monaco Editor pulls in browser-only APIs (clipboard, workers) that jsdom
// does not implement. Stub the editor and its config side-effect module so
// the unit tests can render the app shell without loading the real editor.
vi.mock('@monaco-editor/react', () => ({
  loader: { config: vi.fn() },
  default: vi.fn(() => null),
}));

vi.mock('@/lib/monacoConfig', () => ({
  monaco: {},
}));

export function mockTauriCommand(
  command: string,
  handler: (args?: Record<string, unknown>) => unknown,
): void {
  handlers.set(command, handler);
}

export function clearTauriMocks(): void {
  handlers.clear();
}

afterEach(() => {
  clearTauriMocks();
});
