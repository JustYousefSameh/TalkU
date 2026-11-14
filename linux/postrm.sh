#!/bin/sh
# TalkU post-remove: on a real removal (not an upgrade) drop the state the
# package created. dpkg passes "purge" for `apt purge`, rpm passes "0" for the
# final erase of the last version.
#
#   $1 = "purge" (deb) or "0" (rpm)  -> remove cached config
#   anything else (upgrade/remove)   -> keep it

set -e

case "${1:-}" in
purge | 0)
    # The cached WireGuard config; refetched from the server on next connect.
    if [ -d /var/lib/talku ]; then
        rm -rf -- /var/lib/talku
    fi
    # Packaged daemon + wstunnel (also covers a stale manual install).
    if [ -d /opt/talku ]; then
        rm -rf -- /opt/talku
    fi
    ;;
esac

if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
fi

exit 0
