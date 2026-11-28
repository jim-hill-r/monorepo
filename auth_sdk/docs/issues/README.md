# Notes on auth_sdk's issues

## Native sign-in

Sign-in for desktop and mobile apps (RFC 8252), so cahokia's desktop and
mobile Login buttons work. The plan, its phases and why is in
[docs/plans/2026-11-native-sign-in.md](../plans/2026-11-native-sign-in.md).
Phases 1 (the shared flow and `native::NativeAuthClient`) and 2 (the
desktop listener, `native::loopback`) are done. Phases 3,
4 and 5 live in `infra/docs/issues/`, `cahokia/desktop/docs/issues/` and
`cahokia/mobile/docs/issues/`. Phase 3 is a credential change that waits on Jim's
approval in `infra-credentials`; phase 4 needs phases 2 and 3.

# Epic notes

## Threat check

See [standards/docs/plans/2026-11-threat-check.md](../../../standards/docs/plans/2026-11-threat-check.md)
(approved by Jim, 2026-11-26). One group per phase.
