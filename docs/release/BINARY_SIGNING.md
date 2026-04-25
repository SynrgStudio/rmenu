# BINARY SIGNING — rmenu

Status: research / release policy  
Applies to: Windows zip releases

---

## 1. Current status

Current `rmenu` release artifacts are expected to be unsigned Windows binaries packaged in a zip file.

Current recommendation:

> Ship unsigned zip artifacts with SHA256 checksums for now. Research and adopt code signing before broader public distribution.

Binary signing is not required to complete Wave 0 packaging/release.

---

## 2. What checksums provide

SHA256 checksums help users verify that a downloaded artifact matches the file published by the project.

Checksums help detect:

- corrupted downloads;
- accidental artifact changes;
- mismatched files.

Checksums do not prove:

- publisher identity;
- that Windows trusts the executable;
- that the binary has reputation with SmartScreen.

---

## 3. Windows SmartScreen implications

Unsigned or low-reputation Windows binaries may trigger SmartScreen or browser warnings.

Possible user-facing effects:

- warning on download;
- warning on first run;
- extra confirmation steps;
- lower trust for non-technical users.

This is expected for unsigned small-project binaries and should be documented clearly in install/release notes.

---

## 4. Options

### Option A — Unsigned zip + SHA256 checksums

Pros:

- simple;
- no certificate cost;
- easy to automate;
- sufficient for early technical users.

Cons:

- SmartScreen/browser warnings likely;
- users must trust project/release process manually;
- no publisher identity in Windows properties.

Current fit:

- recommended for initial post-freeze releases.

---

### Option B — Standard code signing certificate

Pros:

- binary has publisher identity;
- improves trust posture;
- can be integrated into CI later.

Cons:

- certificate cost;
- operational complexity;
- private key handling;
- may not immediately remove SmartScreen warnings because reputation builds over time.

Current fit:

- candidate after release process is stable.

---

### Option C — EV code signing certificate

Pros:

- strongest Windows publisher identity story;
- may improve SmartScreen reputation faster.

Cons:

- higher cost;
- stricter identity validation;
- hardware token or managed signing flow may be required;
- more complex CI integration.

Current fit:

- not needed for Wave 0.

---

### Option D — CI-based signing later

Possible future setup:

- GitHub Actions builds release artifact;
- signing key/certificate is stored in a secure signing service or protected secret;
- workflow signs `rmenu.exe` and `rmenu-module-host.exe` before packaging;
- workflow publishes signed zip and checksums.

Risks:

- secret exposure;
- certificate lifecycle management;
- signing workflow maintenance.

Current fit:

- research item for later public distribution.

---

## 5. Current release policy

For the current release line:

- release artifacts may be unsigned;
- publish SHA256 checksums;
- document the unsigned status;
- avoid auto-updaters until trust/signing/update policy is clearer;
- revisit signing before wider public/non-technical distribution.

---

## 6. User guidance for unsigned releases

Recommended install docs should tell users to:

1. download from the official GitHub Releases page;
2. verify SHA256 checksum;
3. extract to a user-controlled folder;
4. run `rmenu.exe --metrics` or `rmenu.exe --modules-debug` as a smoke check;
5. only install modules from sources they trust.

---

## 7. Future research questions

- Which certificate provider is acceptable for the project?
- Is standard code signing enough, or is EV signing needed?
- How will keys be stored and rotated?
- Should signing happen locally or in CI?
- What release threshold justifies signing cost?
- Should Scoop/winget distribution happen before or after signing?
