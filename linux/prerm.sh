#!/bin/sh
# TalkU pre-remove: stop the daemon before its files go away.
#
#   dpkg: prerm remove | prerm upgrade <version>
#   rpm:  %preun 0 (erasing) | %preun 1 (upgrading)
#
# The service is stopped in both cases so the old binary is not left running
# against replaced files, but it is only disabled when the package is really
# being removed: an upgrade must not undo an admin's decision to keep it
# disabled, and it must not leave the unit disabled if %post then fails.

set -e

UNIT=talku-cli.service

if command -v systemctl >/dev/null 2>&1; then
    # ExecStopPost runs `talku-cli down`, so the tunnel and wstunnel child are
    # torn down here rather than left half-open.
    systemctl stop "$UNIT" || true
    case "${1:-}" in
    remove | 0)
        systemctl disable "$UNIT" || true
        ;;
    esac
fi

exit 0
