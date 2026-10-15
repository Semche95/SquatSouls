// Runs the Tauri CLI, with the GStreamer selection of gstreamer-plugins.sh for the Linux builds: the bundler reads it
// from its own environment, so beforeBuildCommand cannot provide it.
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

if (process.platform === 'linux' && process.argv[2] === 'build' && !process.env.GSTREAMER_PLUGINS_DIR) {
  const script = fileURLToPath(new URL('gstreamer-plugins.sh', import.meta.url));
  for (const line of execFileSync(script, { encoding: 'utf8' }).trim().split('\n')) {
    const [name, value] = line.split('=');
    process.env[name] = value;
  }
}

await import('@tauri-apps/cli/tauri.js');
