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

# Set when this run actually changed group membership, so the closing message
# only asks for a logout when the user's session is really out of date.
GROUP_CHANGED=0

add_to_group() {
    _user=$1
    [ -n "$_user" ] || return 0
    [ "$_user" = root ] && return 0
    if id -nG "$_user" 2>/dev/null | tr ' ' '\n' | grep -qx talku; then
        return 0
    fi
    if usermod -aG talku "$_user"; then
        echo "TalkU: added '$_user' to the 'talku' group."
        GROUP_CHANGED=1
    else
        echo "TalkU: could not add '$_user' to the 'talku' group." >&2
    fi
}

# Users with a local desktop login. PackageKit (GNOME Software, KDE Discover)
# runs maintainer scripts without SUDO_USER, so this is the only reliable way
# to find the person who clicked "Install". Greeter and remote sessions are
# excluded: they cannot run the app, and adding them would be surprising.
desktop_users() {
    command -v loginctl >/dev/null 2>&1 || return 0
    loginctl list-sessions --no-legend 2>/dev/null | while read -r _sid _rest; do
        case "$_sid" in
            '' | *[!0-9]*) continue ;;
        esac
        _props=$(loginctl show-session "$_sid" 2>/dev/null) || continue
        printf '%s\n' "$_props" | grep -qx 'Remote=no' || continue
        printf '%s\n' "$_props" | grep -qx 'Class=user' || continue
        printf '%s\n' "$_props" | sed -n 's/^Name=//p'
    done | sort -u
}

# 1. Shared group: the root daemon and the user's app both need it.
if ! getent group talku >/dev/null 2>&1; then
    groupadd --system talku
fi

# 2. Shared state directory: the daemon reads the cached WireGuard config,
#    the non-root app writes it. Group-writable + setgid so new files keep the
#    talku group.
install -d -m 2770 -o root -g talku "$STATE_DIR"

# 3. The bundle carries the daemon with its build-time mode; force the layout
#    the service expects (and keep it root-owned so the non-root app cannot
#    swap what the root service executes).
if [ ! -x "$DAEMON_DIR/talku-cli" ]; then
    echo "TalkU: missing $DAEMON_DIR/talku-cli" >&2
    exit 1
fi
chown root:root "$DAEMON_DIR/talku-cli"
chmod 0755 "$DAEMON_DIR/talku-cli"

# 4. Let the installing desktop user reach the daemon. sudo exports
#    SUDO_USER; PackageKit/GNOME Software do not, so fall back to the users
#    with an active local session.
_install_users=""
if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
    _install_users="$SUDO_USER"
elif command -v loginctl >/dev/null 2>&1; then
    _install_users=$(desktop_users)
fi

for _user in $_install_users; do
    add_to_group "$_user"
done

if [ "$GROUP_CHANGED" -eq 1 ]; then
    echo "TalkU: log out and back in (or reboot) before starting TalkU,"
    echo "TalkU: so your desktop session picks up the new group."
elif [ -z "$_install_users" ]; then
    echo "TalkU: could not determine your desktop user automatically."
    echo "TalkU: add it to the 'talku' group with:"
    echo "TalkU:   sudo usermod -aG talku \"\$USER\""
    echo "TalkU: then log out and back in (or reboot) before starting TalkU."
fi

# 5. Start (or restart, on upgrade) the daemon under systemd.
if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
    if systemctl is-enabled --quiet "$UNIT" 2>/dev/null; then
        systemctl restart "$UNIT" || true
    else
        systemctl enable --now "$UNIT" || true
    fi

    # 6. NetworkManager must not assume wg0 (see 99-talku-unmanaged.conf);
    #    reload rather than restart so active connections are not interrupted.
    if [ -f /etc/NetworkManager/conf.d/99-talku-unmanaged.conf ] &&
        systemctl is-active --quiet NetworkManager 2>/dev/null; then
        systemctl reload NetworkManager || true
    fi
fi

exit 0
