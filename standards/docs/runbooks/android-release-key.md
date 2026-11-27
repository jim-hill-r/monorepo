# Make the Android release key

- **Kind:** one-time, optional
- **Status:** open (`issues/human/optional-create-android-release-key-controlled.md`).
  The token-minting workflow may generate it instead (Jim, 2026-11-27).
- **Time:** about 10 minutes
- **Cost:** $0

## What it's for

Signs the nightly controlled_chaos playtest APK with the same key each
night, so a new build installs over the old one. Until then each APK has a
new debug signature and the old one must be uninstalled first.

## Why a person

The key must outlive the repo and its CI: if it's lost, every installed
copy has to be removed. A person keeps the one safe copy.

## Before you start

- A machine with a JDK (`keytool`), or a Codespace.

## Steps

1. `keytool -genkeypair -keystore release.keystore -alias release -keyalg RSA -keysize 2048 -validity 10000`,
   with one password for both the key and the keystore.
2. `base64 -w0 release.keystore` (on macOS `base64 -i release.keystore`).
3. Add the secrets below.
4. Keep `release.keystore` and its password in a password manager, then
   delete any other copy.

## Where secrets go

Repository secrets: `ANDROID_KEYSTORE_BASE64` (the base64 output) and
`ANDROID_KEYSTORE_PASSWORD`.

## Verify

The next nightly APK installs over the previous one without an
uninstall.

## Undo or rotate

A new key means every tester uninstalls once. Overwrite both secrets.
