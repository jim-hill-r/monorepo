---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Two third-party PDFs are committed through Git LFS:

- `resources/security/Crafting CLI with OAuth 2.0 Authentication_ ... Medium.pdf`
  (12 MB), a saved copy of a Medium article that `ciam.md` already links.
- `learning/computer_science/system_design/ByteByteGo/Bytebytego_Big_Archive_System_Design_2023.pdf`,
  a copy of ByteByteGo's material.

They are other people's copyrighted work in a public repo, and every clone
or CI checkout with LFS pulls them, which counts against GitHub's free LFS
storage and bandwidth. Delete both and link the originals from the notes
next to them (`ciam.md` and `ByteByteGo/README.md`).
