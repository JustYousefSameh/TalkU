#!/bin/sh
# TalkU post-install: prepare the shared state, fix up the installed binaries
# and (re)start the root daemon.
#
# Runs from dpkg's postinst / rpm's %post, so it must be non-interactive and
# must not fail the package transaction when systemd is unavailable (chroot,
# container image build, ...).

set -e

DAEMON_DIR=/opt/talku
STATE_DIR=/var/lib/talku
UNIT=talku-cli.service

# 1. Shared group: the root daemon and the user's app both need it.
if ! getent group talku >/dev/null 2>&1; then
    groupadd --system talku
fi

# 2. Shared state directory: the daemon reads the cached WireGuard config,
#    the non-root app writes it. Group-writable + setgid so new files keep the
#    talku group.
install -d -m 2770 -o root -g talku "$STATE_DIR"

# 3. The bundle carries the binaries with their build-time mode; force the
#    layout the daemon expects (and keep them root-owned so the non-root app
#    cannot swap what the root service executes).
chown root:root "$DAEMON_DIR/talku-cli" "$DAEMON_DIR/talku-wstunnel"
chmod 0755 "$DAEMON_DIR/talku-cli" "$DAEMON_DIR/talku-wstunnel"

# 4. Let the user who ran the install reach the daemon. dpkg/rpm leave
#    SUDO_USER in the environment when installed through sudo.
if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
    if ! id -nG "$SUDO_USER" 2>/dev/null | tr ' ' '\n' | grep -qx talku; then
        usermod -aG talku "$SUDO_USER" || true
        echo "TalkU: added '$SUDO_USER' to the 'talku' group."
        echo "TalkU: log out and back in (or run 'newgrp talku') before starting the app."
    fi
fi

# 5. Start (or restart, on upgrade) the daemon under systemd.
if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
    if systemctl is-enabled --quiet "$UNIT" 2>/dev/null; then
        systemctl restart "$UNIT" || true
    else
        systemctl enable --now "$UNIT" || true
    fi
fi

exit 0
