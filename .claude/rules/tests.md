---
paths:
  - "src/**"
  - "src-tauri/src/**"
  - "src-tauri/tests/**"
---

# Tests

- Tests only live in `src/tests/` (interface) and `src-tauri/tests/` (backend): no `*.test.js` file elsewhere in `src/`, no `#[cfg(test)]` module in `src-tauri/src/`.
- The interface tests run in English and check the English texts, test data included. French only appears where the test is about French itself: switching the language, French plurals or formatting, French input the app must recognize.
- The CI also runs the tests on Windows, which refuses to delete an open file: in the backend tests, close every connection and `Store` on a database (`drop`, or a temporary that is not kept) before removing its file.
