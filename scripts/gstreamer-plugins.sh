#!/usr/bin/env bash
# Selects the GStreamer files bundled in the AppImage (bundleMediaFramework): the webview scans every bundled
# plugin at its first sound, at each launch, and freezes meanwhile, so only the sound output ones are kept.
# Prints the variables read by the bundler, for scripts/tauri.js.
set -euo pipefail

# Plugins loaded when a sound plays, and alsa for the systems without a PulseAudio or PipeWire server.
PLUGINS=(coreelements app audioconvert audioresample autodetect pulseaudio alsa)

# Debian puts the plugin scanner apart from the plugins, Arch next to them.
multiarch=/usr/lib/$(uname -m)-linux-gnu
if [ -d "$multiarch/gstreamer-1.0" ]; then
  plugins_dir=$multiarch/gstreamer-1.0
  scanner=$multiarch/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner
else
  plugins_dir=/usr/lib/gstreamer-1.0
  scanner=$plugins_dir/gst-plugin-scanner
fi

# Links to the selected files, copied by the bundler.
link() {
  [ -f "$1" ] || { echo "Missing GStreamer file: $1" >&2; exit 1; }
  ln -s "$1" "$2"
}

out=$(mktemp -d)
mkdir "$out/plugins" "$out/helpers"
for plugin in "${PLUGINS[@]}"; do
  link "$plugins_dir/libgst$plugin.so" "$out/plugins/"
done
link "$scanner" "$out/helpers/"

echo "GSTREAMER_PLUGINS_DIR=$out/plugins"
echo "GSTREAMER_HELPERS_DIR=$out/helpers"
