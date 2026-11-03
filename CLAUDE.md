@AGENTS.md

## Claude Code

- `.claude/hooks/` blocks the commands the rules forbid (push, `gh`, the installed app's database, secrets, staging everything, modifying a migration, discarding uncommitted work), formats edited Rust files with rustfmt and lints edited JavaScript and Vue files with ESLint. When a hook blocks, change the approach, never work around it.
- Hand broad searches or verbose tasks (exploring several files, analyzing logs) to a subagent when only the conclusion matters: it keeps the details in its own context.
- Use the `code-reviewer` agent after a change and the `security-auditor` agent when a change touches the webview, IPC commands, network, share texts, updater or CI.
