#!/bin/sh
set -eu

REPO="doggy8088/now"
BINARY_NAME="now"
INSTALL_DIR="${NOW_INSTALL_DIR:-"$HOME/.local/bin"}"

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Darwin:arm64|Darwin:aarch64)
      printf '%s\n' "aarch64-apple-darwin"
      ;;
    Darwin:x86_64)
      printf '%s\n' "x86_64-apple-darwin"
      ;;
    Linux:x86_64|Linux:amd64)
      printf '%s\n' "x86_64-unknown-linux-gnu"
      ;;
    *)
      fail "unsupported platform: $os/$arch"
      ;;
  esac
}

download() {
  url="$1"
  dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$dest"
  else
    fail "curl or wget is required"
  fi
}

verify_checksum() {
  archive="$1"
  checksum_file="$2"
  expected="$(awk '{print $1}' "$checksum_file")"
  [ -n "$expected" ] || fail "checksum file is empty"

  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s  %s\n' "$expected" "$archive" | sha256sum -c - >/dev/null
  elif command -v shasum >/dev/null 2>&1; then
    printf '%s  %s\n' "$expected" "$archive" | shasum -a 256 -c - >/dev/null
  else
    fail "sha256sum or shasum is required"
  fi
}

target="$(detect_target)"
archive="$BINARY_NAME-$target.tar.xz"
base_url="https://github.com/$REPO/releases/latest/download"

need_cmd tar
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

download "$base_url/$archive" "$tmp_dir/$archive"
download "$base_url/$archive.sha256" "$tmp_dir/$archive.sha256"
verify_checksum "$tmp_dir/$archive" "$tmp_dir/$archive.sha256"

tar -xJf "$tmp_dir/$archive" -C "$tmp_dir"
binary_path="$(find "$tmp_dir" -type f -name "$BINARY_NAME" -perm -111 | head -n 1)"
[ -n "$binary_path" ] || binary_path="$(find "$tmp_dir" -type f -name "$BINARY_NAME" | head -n 1)"
[ -n "$binary_path" ] || fail "archive did not contain $BINARY_NAME"

mkdir -p "$INSTALL_DIR"
cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
chmod 755 "$INSTALL_DIR/$BINARY_NAME"

printf 'Installed %s\n' "$INSTALL_DIR/$BINARY_NAME"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'Add %s to PATH to run %s from any directory.\n' "$INSTALL_DIR" "$BINARY_NAME" ;;
esac
