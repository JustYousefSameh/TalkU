<div align="center">
  
  ![TalkU Banner](https://github.com/user-attachments/assets/6df5065f-ad80-47ce-b7f3-843d5676c5af)

  # TalkU
  **A lightweight, split-tunneling VPN Application dedicated to fixing Voice Chat and Lobby issues in games.**

  ![TalkU Online](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Ftalku.ddns.net%3A8000%2Fconnected_users%2F&query=%24.connected_users&label=Currently%20Connected&color=success&style=for-the-badge&cacheSeconds=60)
![Discord](https://img.shields.io/discord/1333372651237933056?label=Discord&logo=discord&style=for-the-badge)

</div>

https://github.com/user-attachments/assets/1401e5cb-6466-4fb1-9a55-839758931f03

## About TalkU
TalkU is designed to bypass VOIP restrictions without ruining your in-game ping. It uses [WireGuard](https://www.wireguard.com/) and [wstunnel](https://github.com/erebe/wstunnel) for stealthy obfuscation. **Because it only routes VOIP connections, your actual gameplay traffic remains untouched, ensuring zero ping spikes.**

*(Note: v3.0 is a complete rewrite from JavaFX to Tauri, delivering massive performance and UI improvements!)*

## Download & Installation
1. Download the latest version [here](https://github.com/JustYousefSameh/TalkU/releases/latest/download/TalkU.exe).
2. Extract the file and run the installer.
3. Connect and just enjoy your game!

## Linux
TalkU on Linux is two pieces that have to be installed together:

* the desktop app (this repository's Tauri frontend), which runs as your user, and
* a root systemd daemon, `talku-cli`, which creates the WireGuard interface and runs the WebSocket tunnel.

The daemon needs `wstunnel`, and it must be the exact version TalkU pins: a distro `wstunnel` is often older or patched differently, and a newer one can change CLI flags. The packages therefore ship it **next to the daemon** at `/opt/talku/talku-wstunnel`, which the daemon resolves before ever looking at `PATH`:

```
/opt/talku/talku-cli          root:root 0755   daemon, started by systemd
/opt/talku/talku-wstunnel     root:root 0755   pinned wstunnel client
/lib/systemd/system/talku-cli.service
/var/lib/talku/talkuwg.conf   root:talku 0600  cached WireGuard config (shared)
/var/lib/talku/                root:talku 2770
```

### Build a package
```bash
# Debian/Ubuntu build dependencies (equivalent names exist for rpm distros)
sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libssl-dev libasound2-dev

bun install
bun run prepare:linux          # builds talku-cli, stages wstunnel 10.5.1 in src-tauri/binaries/
bun run tauri build --bundles deb   # or --bundles rpm
```

`bun run prepare:linux` downloads the pinned `wstunnel` release, verifies it against the release's `checksums.txt`, and stages both binaries under `src-tauri/binaries/` (gitignored). Pass `--wstunnel PATH` to use a local build instead; a system `wstunnel` is only reused when its version matches the pin.

The installer scripts then create the `talku` group and add the installing user to it, so the app can write `/var/lib/talku/talkuwg.conf` and reach `/run/talku/talku-cli.sock`:

```bash
sudo apt install ./target/release/bundle/deb/TalkU_*.deb
# or: sudo dnf install ./target/release/bundle/rpm/TalkU-*.rpm
```

AppImage is not built: it cannot install a root-owned daemon, and an AppImage without the daemon cannot connect.

### Install without a package manager
On distributions without `dpkg`/`rpm` (Arch, source installs) use the script, which installs the same layout under `/etc/systemd/system`:

```bash
bun install
sudo scripts/install-linux.sh              # builds the daemon, installs /opt/talku, enables the service
sudo scripts/install-linux.sh --uninstall  # stop, disable and remove everything again
```

### After installing
* **Log out and back in once.** The `talku` group membership only applies to new sessions (`newgrp talku` works for a single terminal, not for the tray app).
* Check the daemon with `systemctl status talku-cli` and `journalctl -u talku-cli -f`.
* The **Launch on startup** toggle writes an XDG autostart entry to `~/.config/autostart/`. GNOME, KDE and XFCE honour it; bare Wayland/X11 window managers (Hyprland, sway, i3) do not — start `talku --autostart` from the compositor config instead.

## Supported Games
TalkU integrates smoothly with games utilizing Vivox and other major voice engines. [View the full Vivox integration list here](https://en.wikipedia.org/wiki/Vivox#Integrations).

### Fixes Lobby Connection Issues:
* Helldivers 2
* GTA V (Epic & Steam Connections)
* Fallout 76 

### Fixes In-Game Voice Chat:
* Valorant
* Overwatch 2
* League Of Legends
* Foxhole
* Ready or Not
* The Hunt: Showdown
* The Cycle: Frontier
* PUBG (Steam)
* Hell Let Loose
* Paladins
* Smite
* The Outlast Trials

## Community & Support
Having trouble connecting or want to request a new game? Come hang out with us!
* **TalkU Discord:** [Join here](https://discord.gg/V4GT6vQv93) for support, updates, and matchmaking.

## Credits & Special Thanks
* Special Thanks to **Axi** — we took heavy inspiration from his scripts. Join his [Discord community here](https://discord.gg/ugFHJUypUS).
* Built with [Tauri](https://tauri.app/).
