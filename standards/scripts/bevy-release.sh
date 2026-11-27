#!/usr/bin/env bash
# Build a Bevy game for players to try: a web build, a macOS app or an
# Android APK. Run from the game's directory; the bevy-game tasks
# release:web, release:macos and release:android call it.
#
#   bevy-release.sh web      dist/web/ (index.html, boot.js, game.js,
#                            game_bg.wasm, and _headers from web/_headers),
#                            ready for `wrangler pages deploy`. Needs the
#                            wasm32-unknown-unknown target, wasm-bindgen-cli
#                            matching the wasm-bindgen in Cargo.lock, and a
#                            [profile.web] in Cargo.toml.
#   bevy-release.sh macos    dist/<name>-macos.zip, a universal (Apple
#                            silicon and Intel) .app, ad-hoc signed. Runs on
#                            macOS only.
#   bevy-release.sh android  dist/<name>-android.apk, built with cargo-apk
#                            from the game's cdylib. Needs ANDROID_HOME and
#                            ANDROID_NDK_ROOT (GitHub's Ubuntu runners set
#                            both). Signed with the key in
#                            ANDROID_KEYSTORE_BASE64 and
#                            ANDROID_KEYSTORE_PASSWORD when they are set,
#                            otherwise with a debug key.
#
# GAME_TITLE (from the game's mise.toml [env]) names the app and the page;
# it defaults to the Cargo package name.
set -euo pipefail

die() { echo "bevy-release: $*" >&2; exit 1; }

# Cloudflare Pages serves files up to 25 MiB (standards/docs/free-tiers.md).
PAGES_MAX_FILE_BYTES=$((25 * 1024 * 1024))

name=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')
title=${GAME_TITLE:-$name}

web() {
  grep -q '^\[profile\.web\]' Cargo.toml ||
    die "Cargo.toml needs a [profile.web] (inherits = \"release\", opt-level = \"s\") so the wasm stays under the Pages file limit"
  cargo build --profile web --target wasm32-unknown-unknown
  rm -rf dist/web
  mkdir -p dist/web
  wasm-bindgen --no-typescript --target web --out-dir dist/web --out-name game \
    --remove-name-section --remove-producers-section \
    "target/wasm32-unknown-unknown/web/$name.wasm"
  cat >dist/web/index.html <<EOF
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>$title</title>
  <style>
    html, body { margin: 0; height: 100%; background: #111; color: #ddd; font-family: system-ui, sans-serif; }
    canvas { display: block; width: 100%; height: 100%; outline: none; }
    #loading { position: fixed; inset: 0; display: grid; place-items: center; }
  </style>
</head>
<body>
  <div id="loading">Loading $title…</div>
  <canvas id="bevy" tabindex="0"></canvas>
  <script type="module" src="./boot.js"></script>
</body>
</html>
EOF
  # The start-up code is a file, not an inline script, so the page works
  # under a Content-Security-Policy without 'unsafe-inline'.
  cat >dist/web/boot.js <<'EOF'
import init from "./game.js";
const canvas = document.getElementById("bevy");
const loading = document.getElementById("loading");
// Bevy's winit loop throws a control-flow exception on purpose once it
// takes over; anything else is a real failure.
init().catch((e) => {
  if (!String(e).includes("Using exceptions for control flow")) {
    loading.textContent = "The game failed to start: " + e;
    throw e;
  }
});
// The game takes keyboard input, so give the canvas focus once it shows.
new MutationObserver(() => { loading.remove(); canvas.focus(); })
  .observe(canvas, { attributes: true });
EOF
  # A game's web/_headers (Cloudflare Pages response headers, such as its
  # Content-Security-Policy) goes in the build output's root.
  if [ -f web/_headers ]; then
    cp web/_headers dist/web/_headers
  fi
  local f size
  for f in dist/web/*; do
    size=$(wc -c <"$f")
    [ "$size" -le "$PAGES_MAX_FILE_BYTES" ] ||
      die "$f is $size bytes; Cloudflare Pages serves files up to 25 MiB"
  done
  echo "bevy-release: web build in dist/web"
  ls -l dist/web
}

macos() {
  [ "$(uname -s)" = Darwin ] || die "the macOS build runs on macOS (the Nightly downloads workflow uses a macos runner)"
  local t
  for t in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build --release --target "$t"
  done
  local app="dist/macos/$title.app"
  rm -rf dist/macos "dist/$name-macos.zip"
  mkdir -p "$app/Contents/MacOS"
  lipo -create -output "$app/Contents/MacOS/$name" \
    "target/aarch64-apple-darwin/release/$name" \
    "target/x86_64-apple-darwin/release/$name"
  local version
  version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
  cat >"$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$title</string>
  <key>CFBundleDisplayName</key><string>$title</string>
  <key>CFBundleIdentifier</key><string>com.jimhillr.$name</string>
  <key>CFBundleExecutable</key><string>$name</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
  # Ad-hoc signature only: Developer ID signing and notarization need a paid
  # Apple Developer account, which the free-tiers rule rules out. Players
  # open it once with System Settings > Privacy & Security > Open Anyway.
  codesign --force --deep --sign - "$app"
  ditto -c -k --keepParent "$app" "dist/$name-macos.zip"
  echo "bevy-release: macOS app in dist/$name-macos.zip"
}

android() {
  [ -n "${ANDROID_HOME:-}" ] || die "ANDROID_HOME is not set (the Android SDK)"
  [ -n "${ANDROID_NDK_ROOT:-}" ] || die "ANDROID_NDK_ROOT is not set (the Android NDK)"
  if [ -n "${ANDROID_KEYSTORE_BASE64:-}" ]; then
    local ks="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/release.keystore"
    base64 --decode <<<"$ANDROID_KEYSTORE_BASE64" >"$ks"
    export CARGO_APK_RELEASE_KEYSTORE="$ks"
    export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="${ANDROID_KEYSTORE_PASSWORD:?ANDROID_KEYSTORE_PASSWORD is not set}"
    echo "bevy-release: signing with the release key"
  else
    # cargo-apk insists on a release keystore for --release, so make a
    # throwaway one, as Android's own debug keystore would be.
    local ks="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/debug.keystore"
    rm -f "$ks"
    keytool -genkeypair -noprompt -keystore "$ks" -storepass android -keypass android \
      -alias androiddebugkey -keyalg RSA -keysize 2048 -validity 10000 \
      -dname "CN=Android Debug,O=Android,C=US" >/dev/null
    export CARGO_APK_RELEASE_KEYSTORE="$ks"
    export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android
    echo "bevy-release: no ANDROID_KEYSTORE_BASE64, so the APK is signed with a new debug key (uninstall the old build before installing a new one)"
  fi
  cargo apk build --release --lib
  local apk
  apk=$(find target/release/apk -maxdepth 1 -name '*.apk' | head -n1)
  [ -n "$apk" ] || die "cargo apk wrote no APK"
  mkdir -p dist
  cp "$apk" "dist/$name-android.apk"
  echo "bevy-release: Android APK in dist/$name-android.apk"
}

case "${1:-}" in
  web) web ;;
  macos) macos ;;
  android) android ;;
  *) die "usage: bevy-release.sh web|macos|android" ;;
esac
