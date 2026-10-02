#!/usr/bin/env bash
# ==============================================================================
#  🌸 Kuroko (黒子) - Remote Binary Installer
#  Usage: curl -fsSL https://raw.githubusercontent.com/Praveensenpai/kuroko/main/install.sh | bash
# ==============================================================================
set -euo pipefail
IFS=$'\n\t'

REPO="Praveensenpai/kuroko"
BINARY="kuroko"
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

echo "🌸 ========================================= 🌸"
echo "    Kuroko (黒子) Declarative Bot Manager     "
echo "🌸 ========================================= 🌸"

# ── Detect architecture ────────────────────────────────────────────────────────
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64|amd64)   TARGET="x86_64-unknown-linux-gnu" ;;
    *)
        echo "❌ Architecture ${ARCH} is not currently supported in prebuilt releases."
        echo "   Please build from source: cargo install --path ."
        exit 1
        ;;
esac

# ── Fetch latest release tag ───────────────────────────────────────────────────
echo "==> Fetching latest release..."
TAG="$(curl -4 -sSL \
    -H "Cache-Control: no-cache" \
    -H "Pragma: no-cache" \
    "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
    | sed -n 's/.*"tag_name":[ ]*"\([^"]*\)".*/\1/p' \
    || true)"

if [ -z "${TAG:-}" ]; then
    echo "❌ Could not fetch latest release tag from GitHub."
    echo "   Visit https://github.com/${REPO}/releases to download manually."
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${BINARY}-${TARGET}.tar.gz"
echo "==> Downloading ${BINARY} ${TAG} (${TARGET})..."

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

if ! curl -4 -fsSL "${DOWNLOAD_URL}" | tar -xz -C "${TMP_DIR}" 2>/dev/null; then
    echo "❌ Failed to download release binary from:"
    echo "   ${DOWNLOAD_URL}"
    echo "   Visit https://github.com/${REPO}/releases to download manually."
    exit 1
fi

install -m 755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
echo "✔ Installed ${BINARY} ${TAG} to ${INSTALL_DIR}/${BINARY}"

# ── Ensure ~/.local/bin is in PATH ─────────────────────────────────────────────
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) export PATH="${INSTALL_DIR}:${PATH}" ;;
esac

if [ -f "${HOME}/.bashrc" ] && ! grep -q '\.local/bin' "${HOME}/.bashrc"; then
    printf '\n# User local binaries\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "${HOME}/.bashrc"
fi

echo ""
echo "✨ Kuroko installed successfully!"
echo "   Run 'kuroko --help' to get started."
echo "   Run 'kuroko init --output bots.toml' to create your first fleet spec."
