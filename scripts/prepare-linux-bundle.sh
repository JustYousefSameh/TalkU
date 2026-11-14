#!/usr/bin/env bash
# Stage the Linux inputs for `bun run tauri build`:
#
#   src-tauri/binaries/talku-cli-<target-triple>   the elevated daemon
#   src-tauri/binaries/wstunnel-<target-triple>    pinned wstunnel client
#
# Both are referenced by bundle.linux.{deb,rpm}.files in tauri.conf.json, which
# installs them side by side into /opt/talku — the directory the daemon resolves
# `talku-wstunnel` from.
#
# wstunnel is taken from --wstunnel, then $WSTUNNEL_BIN, then a system
# `wstunnel` on PATH, and finally downloaded from the pinned upstream release.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"
WSTUNNEL_VERSION="${WSTUNNEL_VERSION:-10.5.1}"

WSTUNNEL_SRC=""
FORCE=0

usage() {
    cat <<'EOF'
usage: scripts/prepare-linux-bundle.sh [--wstunnel PATH] [--force]

  --wstunnel PATH   use this wstunnel binary instead of PATH/the download
  --force           stage binaries even if the host triple is not
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
    --wstunnel)
        [ $# -ge 2 ] || die "--wstunnel needs a path"
        WSTUNNEL_SRC="$2"
        shift 2
        ;;
    --wstunnel=*)
        WSTUNNEL_SRC="${1#*=}"
        shift
        ;;
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
       x86_64-unknown-linux-gnu binaries, but this host targets $TRIPLE.

       Either update bundle.linux.{deb,rpm}.files in tauri.conf.json to the
       "$TRIPLE" names, or re-run with --force to stage them anyway.
EOF
    exit 1
fi

wstunnel_asset_arch() {
    case "$(uname -m)" in
    x86_64 | amd64) echo amd64 ;;
    aarch64 | arm64) echo arm64 ;;
    *) die "unsupported machine '$(uname -m)'; pass --wstunnel PATH" ;;
    esac
}

wstunnel_version_of() {
    "$1" --version 2>/dev/null | awk '{print $NF}'
}

download_wstunnel() {
    local asset="wstunnel_${WSTUNNEL_VERSION}_linux_$(wstunnel_asset_arch).tar.gz"
    local base="https://github.com/erebe/wstunnel/releases/download/v${WSTUNNEL_VERSION}"
    local dir="$ROOT/target/wstunnel-${WSTUNNEL_VERSION}"

    if [ ! -x "$dir/wstunnel" ]; then
        command -v curl >/dev/null 2>&1 || return 1
        mkdir -p "$dir"
        echo "==> downloading wstunnel $WSTUNNEL_VERSION" >&2
        curl -fsSL "$base/$asset" -o "$dir/$asset" || return 1
        curl -fsSL "$base/checksums.txt" -o "$dir/checksums.txt" || return 1
        # The release ships a goreleaser checksums file; verify before use.
        (cd "$dir" && sha256sum --check --ignore-missing checksums.txt >/dev/null) ||
            die "checksum verification failed for $asset"
        tar -xzf "$dir/$asset" -C "$dir" wstunnel
        # The release tarball stores the binary without the exec bit.
        chmod 0755 "$dir/wstunnel"
    fi

    printf '%s\n' "$dir/wstunnel"
}

resolve_wstunnel() {
    if [ -n "$WSTUNNEL_SRC" ]; then
        [ -x "$WSTUNNEL_SRC" ] || die "--wstunnel: '$WSTUNNEL_SRC' is not executable"
        printf '%s\n' "$WSTUNNEL_SRC"
        return
    fi
    if [ -n "${WSTUNNEL_BIN:-}" ]; then
        [ -x "$WSTUNNEL_BIN" ] || die "WSTUNNEL_BIN: '$WSTUNNEL_BIN' is not executable"
        printf '%s\n' "$WSTUNNEL_BIN"
        return
    fi

    # A wstunnel on PATH is only used when it is the version the bundle pins;
    # otherwise download that exact release so the packaged client cannot drift
    # from the server's expectation. PATH stays as the offline fallback.
    local on_path="" version=""
    if on_path="$(command -v wstunnel)"; then
        version="$(wstunnel_version_of "$on_path")"
        if [ "$version" = "$WSTUNNEL_VERSION" ]; then
            printf '%s\n' "$on_path"
            return
        fi
        echo "==> ignoring $on_path ($version): the bundle pins wstunnel $WSTUNNEL_VERSION" >&2
    fi

    local downloaded=""
    if downloaded="$(download_wstunnel)"; then
        printf '%s\n' "$downloaded"
        return
    fi

    [ -n "$on_path" ] ||
        die "no wstunnel $WSTUNNEL_VERSION available: pass --wstunnel PATH, or install\n       wstunnel, or run with network access to fetch the pinned release"
    echo "warning: could not download wstunnel $WSTUNNEL_VERSION; falling back to" >&2
    echo "         $on_path ($version) — the bundle will contain the wrong version" >&2
    printf '%s\n' "$on_path"
}

mkdir -p "$BIN_DIR"

echo "==> building talku-cli (release, $TRIPLE)"
cargo build --release -p talku-cli --manifest-path "$ROOT/Cargo.toml"
install -Dm 0755 "$ROOT/target/release/talku-cli" "$BIN_DIR/talku-cli-$TRIPLE"

WSTUNNEL="$(resolve_wstunnel)"
echo "==> using $("$WSTUNNEL" --version 2>/dev/null || echo "wstunnel at $WSTUNNEL")"
install -Dm 0755 "$WSTUNNEL" "$BIN_DIR/wstunnel-$TRIPLE"

echo
echo "staged:"
ls -l "$BIN_DIR/talku-cli-$TRIPLE" "$BIN_DIR/wstunnel-$TRIPLE"
echo
echo "next: bun run tauri build --bundles deb   # or --bundles rpm"
