---
added: 2026-11-26
---

Optional: create an Android release key for the controlled_chaos playtest APK and add it as the repo secrets `ANDROID_KEYSTORE_BASE64` (the keystore, base64-encoded) and `ANDROID_KEYSTORE_PASSWORD`. For example `keytool -genkeypair -keystore release.keystore -alias release -keyalg RSA -keysize 2048 -validity 10000`, using the same password for the key and the keystore, then `base64 -w0 release.keystore`. Keep a copy of the keystore somewhere safe.

Until then each nightly APK has a new debug signature, so the old build has to be uninstalled before installing the new one.
