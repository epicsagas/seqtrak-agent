#!/usr/bin/env sh
set -eu

REPO="epicsagas/seqtrak-agent"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_PART="apple-darwin" ;;
  Linux)  OS_PART="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS (use install.ps1 on Windows)" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_PART="x86_64" ;;
  arm64|aarch64) ARCH_PART="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH_PART}-${OS_PART}"
ARCHIVE="seqtrak-agent-${TARGET}.tar.gz"
DATA_DIR="${DATA_DIR:-$HOME/.local/share/seqtrak-agent}"
if [ -n "${SEQTRAK_VERSION:-}" ]; then
  URL="https://github.com/${REPO}/releases/download/${SEQTRAK_VERSION}/${ARCHIVE}"
else
  URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${URL}..."
curl -fsSL "$URL" -o "${TMP_DIR}/${ARCHIVE}"
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
install -m 755 "${TMP_DIR}/st" "${INSTALL_DIR}/st"
install -m 755 "${TMP_DIR}/st-mcp" "${INSTALL_DIR}/st-mcp"

mkdir -p "$DATA_DIR"
if [ -d "${TMP_DIR}/gui" ]; then
  rm -rf "${DATA_DIR}/gui"
  cp -r "${TMP_DIR}/gui" "${DATA_DIR}/gui"
fi
for f in LICENSE NOTICE THIRD_PARTY_LICENSES.md THIRD_PARTY_LICENSES_GUI.md; do
  [ -f "${TMP_DIR}/${f}" ] && cp "${TMP_DIR}/${f}" "${DATA_DIR}/${f}"
done

if [ -d "${TMP_DIR}/.claude/skills/seqtrak" ]; then
  mkdir -p "$HOME/.claude/skills"
  rm -rf "$HOME/.claude/skills/seqtrak"
  cp -r "${TMP_DIR}/.claude/skills/seqtrak" "$HOME/.claude/skills/seqtrak"
  if [ -d "$HOME/.codex" ]; then
    mkdir -p "$HOME/.codex/skills"
    rm -rf "$HOME/.codex/skills/seqtrak"
    cp -r "${TMP_DIR}/.claude/skills/seqtrak" "$HOME/.codex/skills/seqtrak"
  fi
fi

echo "Installed st and st-mcp to ${INSTALL_DIR}"
echo "Installed web GUI and license files to ${DATA_DIR}"
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Add ${INSTALL_DIR} to your PATH if needed:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
