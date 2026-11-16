#!/usr/bin/env bash
# Install (or remove) TalkU's Linux runtime without a package manager:
#
#   /opt/talku/talku-cli         elevated daemon (wstunnel is linked in-process)
#   /etc/systemd/system/talku-cli.service
#
# Equivalent to installing the .deb/.rpm on systems without dpkg/rpm (Arch,
# source installs). Building the daemon needs a Rust toolchain.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
DAEMON_DIR=/opt/talku
UNIT_PATH=/etc/systemd/system/talku-cli.service
SERVICE=talku-cli.service

MODE=install
DAEMON_SRC=""
BUILD=1

usage() {
    cat <<'EOF'
usage: sudo scripts/install-linux.sh [options]

  --daemon PATH     install this talku-cli instead of building it
  --no-build        do not run cargo (requires --daemon)
  --uninstall       stop the service and remove the installed files
  -h, --help        show this help

The installing user (SUDO_USER) is added to the 'talku' group so the desktop
app can talk to the root daemon.
EOF
}

die() {
    echo "error: $*" >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
    --daemon)
        [ $# -ge 2 ] || die "--daemon needs a path"
        DAEMON_SRC="$2"
        shift 2
        ;;
    --no-build)
        BUILD=0
        shift
        ;;
    --uninstall)
        MODE=uninstall
        shift
        ;;
    -h | --help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        die "unknown argument: $1"
        ;;
    esac
done

[ "$(id -u)" -eq 0 ] || die "run this with sudo (it installs a root systemd service)"

if [ "$MODE" = uninstall ]; then
    echo "==> stopping and removing TalkU"
    sh "$ROOT/linux/prerm.sh"
    rm -f "$UNIT_PATH"
    systemctl daemon-reload 2>/dev/null || true
    sh "$ROOT/linux/postrm.sh" purge
    echo "done (state in /var/lib/talku and files in /opt/talku removed)"
    exit 0
fi

# 1. Resolve the daemon binary.
if [ -n "$DAEMON_SRC" ]; then
    [ -x "$DAEMON_SRC" ] || die "--daemon: '$DAEMON_SRC' is not executable"
else
    [ "$BUILD" -eq 1 ] || die "--no-build needs --daemon PATH"
    echo "==> building talku-cli (release)"
    cargo build --release -p talku-cli --manifest-path "$ROOT/Cargo.toml"
    DAEMON_SRC="$ROOT/target/release/talku-cli"
fi

# 2. Install the daemon and its systemd unit.
echo "==> installing into $DAEMON_DIR"
install -d -m 0755 -o root -g root "$DAEMON_DIR"
install -m 0755 -o root -g root "$DAEMON_SRC" "$DAEMON_DIR/talku-cli"
install -Dm 0644 -o root -g root "$ROOT/linux/talku-cli.service" "$UNIT_PATH"

# 3. Group, state dir, systemd registration — shared with the package scripts.
sh "$ROOT/linux/postinst.sh"

echo
echo "installed:"
echo "  $DAEMON_DIR/talku-cli"
echo "  $UNIT_PATH"
systemctl --no-pager --lines=0 status "$SERVICE" 2>/dev/null || true
echo
echo "If this is your first install, log out and back in so the 'talku' group"
echo "membership applies to your session; then launch TalkU."
