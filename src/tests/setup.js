import { vi } from 'vitest';

// The Tauri backend does not exist under Vitest: each test simulates its responses.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
