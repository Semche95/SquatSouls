---
paths:
  - "src/sharing/**"
  - "src/tests/**"
---

# Share format

- Any change to the share/import format (`share.js`) is versioned: increment `VERSION`, add the new version's reader to `READERS` while keeping those of the previous ones, and test importing texts from the old versions.
