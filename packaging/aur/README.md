# AUR package: `talku-bin`

`packaging/aur/talku-bin` is the source of the AUR package `talku-bin`. It does
not build anything: it unpacks the upstream `.deb` from the GitHub release and
installs the desktop app and the root daemon. wstunnel is linked into the
daemon, so there is no separate executable to install.

The `.deb` is produced by `.github/workflows/build-and-release.yml` whenever a
release is published, so the release asset is always the same build the `.deb`
and `.rpm` users get.

## What the package installs

| Path | Source |
| --- | --- |
| `/usr/bin/talku` | desktop app |
| `/opt/talku/talku-cli` | daemon (wstunnel linked in-process) |
| `/usr/lib/systemd/system/talku-cli.service` | systemd unit |
| `/usr/share/applications/TalkU.desktop`, `/usr/share/icons/hicolor/...` | menu entry and icons |
| `/usr/lib/TalkU/resources/*.wav` | connection audio cues |

`talku-bin.install` creates the `talku` group, `/var/lib/talku` (2770,
setgid), adds the invoking user to the group, and on removal stops and disables
the unit. It does not enable the service — Arch expects that to be explicit:

```sh
systemctl enable --now talku-cli
```

## Cutting a new version

The release tag and `src-tauri/tauri.conf.json` must agree, because the `.deb`
filename (`TalkU_<version>_amd64.deb`) and the PKGBUILD URL are derived from
them. The workflow fails the release job if they differ.

1. Bump `version` in `src-tauri/tauri.conf.json` (and `pkgver` below).
2. Publish a GitHub release tagged `v<version>`.
3. Wait for **Linux packages** to attach `TalkU_<version>_amd64.deb` and
   `TalkU-<version>-1.x86_64.rpm`.
4. Update this directory:

   ```sh
   cd packaging/aur/talku-bin
   sed -i "s/^pkgver=.*/pkgver=<version>/" PKGBUILD
   updpkgsums                 # replaces the placeholder SKIP with the real sha256
   makepkg --printsrcinfo > .SRCINFO
   makepkg -f                 # optional: build and inspect before pushing
   ```

5. Commit here, then push to the AUR:

   ```sh
   git clone ssh://aur@aur.archlinux.org/talku-bin.git /tmp/talku-bin
   cp PKGBUILD talku-bin.install .SRCINFO /tmp/talku-bin/
   cd /tmp/talku-bin && git add -A && git commit -m "upgpkg: talku-bin <version>-1" && git push
   ```

## Before the first AUR push

- `sha256sums=('SKIP')` is a placeholder. Run `updpkgsums` after the release
  asset exists — AUR reviewers reject `SKIP` for a released package.
- The repository has no `LICENSE` file, so the package declares
  `license=('custom')`. Add a license (and `/usr/share/licenses/talku-bin/`)
  before submitting; the AUR flags packages without a valid license.
- Only `x86_64` is published, so the PKGBUILD is `arch=('x86_64')`.
