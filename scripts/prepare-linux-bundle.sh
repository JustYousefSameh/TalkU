#!/usr/bin/env bash
# Stage the Linux input for `bun run tauri build`:
#
#   src-tauri/binaries/talku-cli-<target-triple>   the elevated daemon
#
# bundle.linux.{deb,rpm}.files in tauri.conf.json installs it into /opt/talku.
# The pinned wstunnel client is linked into talku-cli, so there is no sibling
# executable to stage or download.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"

FORCE=0

usage() {
    cat <<'EOF'
usage: scripts/prepare-linux-bundle.sh [--force]

  --force           stage the binary even if the host triple is not
                    x86_64-unknown-linux-gnu (then update tauri.conf.json)
  -h, --help        show this help
EOF
}

die() {
    echo "error: $*" >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
    --force)
        FORCE=1
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

TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
[ -n "$TRIPLE" ] || die "could not determine the host target triple from rustc"

if [ "$TRIPLE" != "x86_64-unknown-linux-gnu" ] && [ "$FORCE" -eq 0 ]; then
    cat >&2 <<EOF
error: src-tauri/tauri.conf.json points the Linux bundle at the
       x86_64-unknown-linux-gnu binary, but this host targets $TRIPLE.

       Either update bundle.linux.{deb,rpm}.files in tauri.conf.json to the
       "$TRIPLE" names, or re-run with --force to stage it anyway.
EOF
    exit 1
fi

mkdir -p "$BIN_DIR"

echo "==> building talku-cli (release, $TRIPLE)"
cargo build --release -p talku-cli --manifest-path "$ROOT/Cargo.toml"
install -Dm 0755 "$ROOT/target/release/talku-cli" "$BIN_DIR/talku-cli-$TRIPLE"

echo
echo "staged:"
ls -l "$BIN_DIR/talku-cli-$TRIPLE"
echo
echo "next: bun run tauri build --bundles deb   # or --bundles rpm"
