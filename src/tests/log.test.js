import { beforeEach, describe, expect, it, vi } from 'vitest';
import { error } from '@tauri-apps/plugin-log';
import { logError, logUncaught } from '../log.js';

// Messages sent to the backend's log file.
const logged = () => error.mock.calls.map(([message]) => message);

describe('interface errors', () => {
  beforeEach(() => {
    error.mockClear();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  it('are written to the log file with their context', () => {
    logError('Native texts not sent', 'no backend');
    expect(logged()).toEqual(['Native texts not sent: no backend']);
  });

  it('are logged when uncaught or unhandled', () => {
    const target = new EventTarget();
    logUncaught(target);
    target.dispatchEvent(Object.assign(new Event('error'), { message: 'boom' }));
    target.dispatchEvent(Object.assign(new Event('unhandledrejection'), { reason: 'lost' }));
    expect(logged()).toEqual(['Uncaught error: boom', 'Unhandled rejection: lost']);
  });
});
