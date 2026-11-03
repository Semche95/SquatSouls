#!/usr/bin/env bash
# PostToolUse hook on Edit and Write: formats edited Rust files with rustfmt (as cargo fmt does)
# and lints edited JavaScript and Vue files with ESLint. Exit code 2 shows the lint errors to Claude.

file=$(jq -r '.tool_input.file_path // empty')
[ -f "$file" ] || exit 0
root="$CLAUDE_PROJECT_DIR"
rel="${file#"$root"/}"

case "$rel" in
  src-tauri/*.rs)
    out=$(rustfmt --edition 2021 "$file" 2>&1) || { echo "rustfmt failed on $rel:" >&2; echo "$out" >&2; exit 2; }
    ;;
  src/*.js | src/*.vue | site/*.js | site/*.vue | scripts/*.js | [!/]*.config.js)
    case "$rel" in */*.config.js) case "$rel" in src/* | site/* | scripts/*) ;; *) exit 0 ;; esac ;; esac
    out=$(cd "$root" && pnpm exec eslint --no-warn-ignored "$rel" 2>&1) || { echo "ESLint errors in $rel:" >&2; echo "$out" >&2; exit 2; }
    ;;
esac
exit 0
