---
added: 2026-11-26
unblocks: "the OpenTofu state Worker and the infra plan and apply workflows (decisions 0007 and 0009)"
---

Make two long random passwords for the OpenTofu state Worker (decision 0009), for example with a password manager's generator (32 or more characters each), and add them in GitHub: `TOFU_STATE_READ_PASSWORD` as a repository secret (PR plans only read state), and `TOFU_STATE_WRITE_PASSWORD` only in the `infra` environment (limited to `main`, from `auth0-iac-credentials.md`). Don't keep a copy anywhere else: the Worker's deploy copies them into the Worker from GitHub. Cost: $0, about 5 minutes.
