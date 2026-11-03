#!/usr/bin/env bash
# PreToolUse hook on Bash: blocks dangerous commands and those the project's rules forbid.
# Exit code 2 blocks the command and shows the reason to Claude.

cmd=$(jq -r '.tool_input.command // empty')
[ -z "$cmd" ] && exit 0

block() {
  echo "Blocked by .claude/hooks/validate-bash.sh: $1" >&2
  exit 2
}

sep='(^|[;&|()`[:space:]])'
# Global git options before the subcommand (git -C dir, git -c key=value, git --no-pager).
opts='((-C|-c)[[:space:]]+[^[:space:]]+[[:space:]]+|--[a-z-]+(=[^[:space:]]+)?[[:space:]]+)*'

# The user pushes themselves.
grep -qE "${sep}git[[:space:]]+${opts}push([[:space:]]|$)" <<<"$cmd" &&
  block 'never push, the user pushes themselves.'

# No GitHub CLI: ask the user for the information.
grep -qE "${sep}gh([[:space:]]|$)" <<<"$cmd" &&
  block 'never use the gh CLI, ask the user for the information.'

# The production database is never read, modified, copied or deleted (squatsouls-dev.db is allowed).
grep -qE 'squatsouls\.(db|previous\.db|damaged)' <<<"$cmd" &&
  block 'never touch the production database, only squatsouls-dev.db or test databases.'

# Secrets: the updater signing key and env files.
grep -qE '\.tauri/|squatsouls\.key|(^|[/[:space:]])\.env([.[:space:]]|$)' <<<"$cmd" &&
  block 'never read secrets (signing key, .env).'

# Files are staged one by one.
grep -qE "${sep}git[[:space:]]+${opts}add[[:space:]]+([^;&|]*[[:space:]])?(-A|--all|-u|--update|\.|:/)([[:space:]]|$)" <<<"$cmd" &&
  block 'stage files one by one, never all at once.'

# Commits carry no Co-Authored-By and keep the hooks.
if grep -qE "${sep}git[[:space:]]+${opts}commit" <<<"$cmd"; then
  grep -qiE 'co-authored-by' <<<"$cmd" && block 'commits are written without Co-Authored-By.'
  grep -qE '(--no-verify|[[:space:]]-n([[:space:]]|$))' <<<"$cmd" && block 'never skip the git hooks.'
fi

# Uncommitted work is never thrown away.
grep -qE "${sep}git[[:space:]]+${opts}(reset[[:space:]]+([^;&|]*[[:space:]])?--hard|clean[[:space:]]+([^;&|]*[[:space:]])?-[a-zA-Z]*f|checkout[[:space:]]+([^;&|]*[[:space:]])?\.([[:space:]]|$)|restore[[:space:]]+([^;&|]*[[:space:]])?\.([[:space:]]|$)|stash[[:space:]]+(drop|clear))" <<<"$cmd" &&
  block 'this would discard uncommitted changes; check git diff and ask the user.'

# A migration is never modified, renamed or deleted once created.
while read -r file; do
  [ -n "$file" ] && [ -e "$CLAUDE_PROJECT_DIR/$file" ] &&
    grep -qE "${sep}(rm|mv|cp|sed[[:space:]]+-i|perl[[:space:]]+-[a-z]*i|truncate|tee|git[[:space:]]+(rm|mv|checkout|restore))([[:space:]]|$)|>[[:space:]]*[^[:space:]]*$file" <<<"$cmd" &&
    block "never modify, rename or delete $file: add a new V<n+1>__<name>.sql migration."
done < <(grep -oE 'src-tauri/migrations/V[0-9]+__[A-Za-z0-9_]+\.sql' <<<"$cmd")

# Dangerous system commands.
grep -qE "${sep}sudo([[:space:]]|$)" <<<"$cmd" && block 'no sudo.'
grep -qE "${sep}rm[[:space:]]+(-[a-zA-Z]+[[:space:]]+)*-[a-zA-Z]*[rR][a-zA-Z]*[[:space:]]+(-[a-zA-Z]+[[:space:]]+)*(/|~|\\\$HOME|\.\.?)/?\*?([[:space:]]|$)" <<<"$cmd" &&
  block 'recursive deletion of a root, home or whole directory.'
grep -qE "${sep}(mkfs|dd[[:space:]]+[^;&|]*of=/dev/|chmod[[:space:]]+-R[[:space:]]+777)" <<<"$cmd" &&
  block 'dangerous system command.'

exit 0
