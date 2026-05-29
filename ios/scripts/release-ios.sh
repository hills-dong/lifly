#!/usr/bin/env bash
# Archive the Lifly iOS app and upload it to App Store Connect / TestFlight.
#
# Mirrors the DS Music release flow. Requires (paid Apple Developer account):
#   LIFLY_TEAM_ID   10-char Apple Developer Team ID
#   ASC_KEY_ID      App Store Connect API Key ID (10 chars)
#   ASC_ISSUER_ID   App Store Connect API Issuer ID (UUID)
#   key file at ~/.appstoreconnect/private_keys/AuthKey_<ASC_KEY_ID>.p8
# These are auto-sourced from ~/.appstoreconnect/lifly-release.env.
#
# The app record (the iOS bundle id) must already exist in App Store Connect.
# Signing cert + App Store provisioning profile are created automatically via
# the API key (-allowProvisioningUpdates).
#
# Usage:
#   bash ios/scripts/release-ios.sh            # archive + export + upload
#   bash ios/scripts/release-ios.sh --archive  # archive only (registers App ID)
#   bash ios/scripts/release-ios.sh --upload   # export + upload (after app exists)
set -euo pipefail

IOS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$IOS_DIR"

# shellcheck disable=SC1090
[ -f "$HOME/.appstoreconnect/lifly-release.env" ] && . "$HOME/.appstoreconnect/lifly-release.env"

: "${LIFLY_TEAM_ID:?set LIFLY_TEAM_ID}"
: "${ASC_KEY_ID:?set ASC_KEY_ID}"
: "${ASC_ISSUER_ID:?set ASC_ISSUER_ID}"

P8="$HOME/.appstoreconnect/private_keys/AuthKey_${ASC_KEY_ID}.p8"
[ -f "$P8" ] || { echo "missing API key: $P8"; exit 1; }

MODE="${1:-all}"
ARCHIVE=".build/Lifly.xcarchive"
EXPORT=".build/export"
OPTS=".build/ExportOptions.plist"
BUILD_NO="$(date +%Y%m%d%H%M)"   # monotonic, unique per upload

do_archive() {
  echo "release-ios: regenerating Xcode project"
  xcodegen generate >/dev/null
  echo "release-ios: archiving (build $BUILD_NO)"
  rm -rf "$ARCHIVE" "$EXPORT"
  xcodebuild archive \
    -project Lifly.xcodeproj -scheme Lifly \
    -configuration Release \
    -destination 'generic/platform=iOS' \
    -archivePath "$ARCHIVE" \
    -allowProvisioningUpdates \
    -authenticationKeyID "$ASC_KEY_ID" \
    -authenticationKeyIssuerID "$ASC_ISSUER_ID" \
    -authenticationKeyPath "$P8" \
    DEVELOPMENT_TEAM="$LIFLY_TEAM_ID" \
    CODE_SIGN_STYLE=Automatic \
    CURRENT_PROJECT_VERSION="$BUILD_NO" \
    | tail -4
}

do_export_upload() {
  [ -d "$ARCHIVE" ] || { echo "no archive at $ARCHIVE — run --archive first"; exit 1; }
  mkdir -p .build
  sed "s/__TEAM_ID__/$LIFLY_TEAM_ID/" Packaging/ExportOptions.plist > "$OPTS"

  echo "release-ios: exporting .ipa"
  rm -rf "$EXPORT"
  xcodebuild -exportArchive \
    -archivePath "$ARCHIVE" \
    -exportPath "$EXPORT" \
    -exportOptionsPlist "$OPTS" \
    -allowProvisioningUpdates \
    -authenticationKeyID "$ASC_KEY_ID" \
    -authenticationKeyIssuerID "$ASC_ISSUER_ID" \
    -authenticationKeyPath "$P8" \
    | tail -4

  IPA="$(/usr/bin/find "$EXPORT" -name '*.ipa' | head -1)"
  [ -n "$IPA" ] || { echo "no .ipa produced"; exit 1; }
  echo "release-ios: uploading $IPA"
  xcrun altool --upload-app -f "$IPA" -t ios \
    --apiKey "$ASC_KEY_ID" --apiIssuer "$ASC_ISSUER_ID"
  echo "release-ios: done — build uploaded; appears in TestFlight after ~5-15 min."
}

case "$MODE" in
  --archive) do_archive ;;
  --upload)  do_export_upload ;;
  all)       do_archive; do_export_upload ;;
  *) echo "unknown mode: $MODE (use --archive | --upload | omit for all)"; exit 1 ;;
esac
