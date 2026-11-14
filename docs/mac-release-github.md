# Building the notarized Mac disk image on GitHub

[`.github/workflows/release-mac.yml`](../.github/workflows/release-mac.yml) builds
`Clipiccolo-X.Y.Z-mac-arm64.dmg` on a GitHub macOS runner with `build-mac.sh`. It signs the app with the Developer ID
certificate, notarizes and staples both the app and the disk image, and attaches the DMG to a **draft** release. It
runs when a `vX.Y.Z` tag is pushed, or by hand from the Actions tab.

The runner has neither your keychain nor a `notarytool` profile, so it gets the certificate and an App Store Connect
API key from five repository secrets. The setup below is a one-time job.

## 1. Export the Developer ID certificate (.p12)

1. Open **Keychain Access** and select keychain **login**, then the tab **My Certificates**.
2. Find **Developer ID Application: … (T6G2PWFUFQ)**. Click its arrow and make sure a **private key** sits underneath.
   Without the key, signing on GitHub fails.
3. Right-click the certificate (not the key) → **Export…** → **Personal Information Exchange (.p12)** →
   `~/Desktop/devid.p12`. Set a strong export password and keep it, because it becomes a secret.
4. Convert it to text:

   ```sh
   base64 -i ~/Desktop/devid.p12 -o ~/Desktop/devid.p12.b64
   ```

## 2. Create an App Store Connect API key (for notarization)

1. Go to <https://appstoreconnect.apple.com> → **Users and Access** → **Integrations** → **App Store Connect API** →
   **Team Keys**. Only the Account Holder or an Admin can do this. The first time, Apple asks you to request access.
2. Click **+**, enter the name `GitHub notarization`, choose access **Developer**, and click **Generate**.
3. Note the **Issuer ID** (top of the page) and the **Key ID** (in the key's row).
4. Download `AuthKey_<KEYID>.p8`. Apple offers the download **only once**, so keep a copy in a password manager.

## 3. Add the repository secrets

On GitHub: **Settings → Secrets and variables → Actions → New repository secret**. Or from the repo folder:

```sh
gh secret set APPLE_CERTIFICATE < ~/Desktop/devid.p12.b64
gh secret set APPLE_CERTIFICATE_PASSWORD     # paste the .p12 export password
gh secret set APPLE_API_ISSUER               # paste the Issuer ID
gh secret set APPLE_API_KEY                  # paste the Key ID
gh secret set APPLE_API_KEY_P8 < ~/Downloads/AuthKey_<KEYID>.p8
gh secret list                               # should list all five
rm ~/Desktop/devid.p12 ~/Desktop/devid.p12.b64
```

Optional: for a manual approval before every release build, create an Actions environment `release` with yourself as
required reviewer. Store the secrets there and add `environment: release` to the job.

## 4. Test run (no tag)

1. Push the workflow to `main`. Manual runs only work for workflows on the default branch.
2. Start the run: **Actions → Release (macOS) → Run workflow**, or `gh workflow run release-mac.yml`. The first run
   takes about 20–25 minutes because it compiles tauri-cli; later runs are cached.
3. Download the `clipiccolo-mac-dmg` artifact **in a browser**, so it gets the quarantine flag that users' downloads
   get. Unzip it, then run:

   ```sh
   spctl -a -t open --context context:primary-signature -vv Clipiccolo-*-mac-arm64.dmg   # accepted, Notarized Developer ID
   xcrun stapler validate Clipiccolo-*-mac-arm64.dmg                                      # The validate action worked!
   ```

   Then open the DMG, drag the app to Applications and start it. No warning should appear.

## 5. Releasing

1. Bump the version as described in [CONTRIBUTING.md](../CONTRIBUTING.md#releasing-maintainer), then commit.
2. Tag and push the tag: `git tag vX.Y.Z && git push origin vX.Y.Z`. The workflow fails if the tag does not match
   `tauri.conf.json`.
3. The DMG appears in the draft release `vX.Y.Z`. Add the Windows files and the FFmpeg source tarballs, then publish.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `MAC verification failed` while importing the certificate | `APPLE_CERTIFICATE_PASSWORD` is wrong |
| Ad-hoc signature, or "no identity found" | The .p12 was exported without its private key (step 1.2) |
| `failed to notarize app: Error: invalidPEMDocument` | The `.p8` secret is mangled (pasted without newlines or `BEGIN`/`END` lines). The workflow now repairs common cases; if it still fails, re-set it from the file: `gh secret set APPLE_API_KEY_P8 < AuthKey_<KEYID>.p8` |
| notarytool `401` / invalid credentials | Issuer ID and Key ID are swapped, or the `.p8` secret is missing its `BEGIN`/`END` lines |
| Notarization status `Invalid` | `xcrun notarytool log <submission-id> --key AuthKey.p8 --key-id … --issuer …` names the rejected file |
| `Tag vX does not match version Y` | Fix the version, then delete and re-push the tag |

## Notes

- The repository is public, so GitHub's standard runners (macOS included) cost nothing.
- Secrets are not passed to workflows triggered from forks, and this workflow only runs on tags or a manual start.
- If the `.p8` leaks, revoke the key in App Store Connect. If the `.p12` leaks, revoke the certificate at
  developer.apple.com and create a new one. Apps that already shipped keep working.
- ffmpeg is pinned by URL and SHA-256 in the workflow. When updating ffmpeg, change it there and locally together.
