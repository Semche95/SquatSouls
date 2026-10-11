import { vi } from 'vitest';

// Le backend Tauri n'existe pas sous Vitest : chaque test simule ses réponses.
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
