# Notes on auth_sdk's issues

## Native sign-in

Sign-in for desktop and mobile apps (RFC 8252), so cahokia's desktop and
mobile Login buttons work. The plan, its phases and why is in
[docs/plans/2026-11-native-sign-in.md](../docs/plans/2026-11-native-sign-in.md).
Phase 1 (the shared flow and `native::NativeAuthClient`) is done. Phases 3,
4 and 5 live in `infra/issues/`, `cahokia/desktop/issues/` and
`cahokia/mobile/issues/`. Phase 3 is a credential change that waits on Jim's
approval in `infra-credentials`; phase 4 needs phases 2 and 3.
