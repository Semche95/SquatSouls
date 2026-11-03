# Git

- Never commit without the user's explicit agreement: accepting an operation that leads to it is not agreement; do the work locally, then ask.
- Never push (`git push`, force push included), even on explicit request: the user pushes themselves.
- Stage files one by one and check `git diff --cached --stat` before each commit.
- Commits in Conventional Commits, a single line, without `Co-Authored-By`.
- Each commit is consistent: a single change, with its code, tests and documentation, and the tests pass. Split by file hunk (not by whole file) when a file mixes several changes.
