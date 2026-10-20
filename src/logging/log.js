import { error } from '@tauri-apps/plugin-log';

// Writes an interface error to the console and to the backend's log file.
export function logError(context, e) {
  // eslint-disable-next-line no-console -- this function is the interface's error output
  console.error(context, e);
  error(`${context}: ${e?.stack ?? e}`).catch(() => {});
}

// Sends every uncaught error and rejection of the page to the log file.
export function logUncaught(target = window) {
  target.addEventListener('error', (event) => logError('Uncaught error', event.error ?? event.message));
  target.addEventListener('unhandledrejection', (event) => logError('Unhandled rejection', event.reason));
}
