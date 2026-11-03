#!/usr/bin/env bash
# PreToolUse hook on Edit and Write: an existing migration is never modified, even uncommitted,
# since refinery checksums its content. Writing a new migration file is allowed.

file=$(jq -r '.tool_input.file_path // empty')
case "$file" in
  */src-tauri/migrations/V*.sql)
    if [ -e "$file" ]; then
      echo "Blocked by .claude/hooks/protect-migrations.sh: never modify $file, add a new V<n+1>__<name>.sql migration." >&2
      exit 2
    fi
    ;;
esac
exit 0
