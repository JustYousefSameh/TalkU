#!/bin/sh
# TalkU pre-remove: stop and disable the daemon before its files go away.

set -e

UNIT=talku-cli.service

if command -v systemctl >/dev/null 2>&1; then
    # ExecStopPost runs `talku-cli down`, so the tunnel and wstunnel child are
    # torn down here rather than left half-open.
    systemctl stop "$UNIT" || true
    systemctl disable "$UNIT" || true
fi

exit 0
