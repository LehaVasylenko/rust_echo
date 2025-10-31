#!/usr/bin/env bash
set -euo pipefail

# -------- args --------
APP_NAME="${1:-myrustapp}"
REPO_URL="${2:-git@github.com:LehaVasylenko/rust_echo.git}"
BRANCH="${3:-main}"
RUN_USER_ARG="${4:-appuser}"

# -------- require root --------
if [[ $EUID -ne 0 ]]; then
  echo "Please run as root: sudo bash $0 <app> <repo> [branch] [run-user]" >&2
  exit 1
fi

# Кто будет БИЛДИТЬ (обычно твой обычный юзер, у которого стоит Rust):
BUILD_USER="${SUDO_USER:-$(logname)}"
# Под кем будет РАБОТАТЬ сервис:
RUN_USER="$RUN_USER_ARG"

APP_DIR="/opt/${APP_NAME}"
ENV_DIR="/etc/${APP_NAME}"
ENV_FILE="${ENV_DIR}/env"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"
SRC_DIR="/tmp/${APP_NAME}-src"

# Найти cargo у BUILD_USER
CARGO_BIN="$(sudo -u "$BUILD_USER" -H bash -lc 'command -v cargo || true')"
if [[ -z "$CARGO_BIN" ]]; then
  if sudo -u "$BUILD_USER" test -x "/home/${BUILD_USER}/.cargo/bin/cargo"; then
    CARGO_BIN="/home/${BUILD_USER}/.cargo/bin/cargo"
  else
    echo "ERROR: cargo not found for user ${BUILD_USER}. Install Rust via rustup for that user." >&2
    exit 1
  fi
fi

# Инструменты, которые нужны root-процессу
need_root_bin() { command -v "$1" >/dev/null 2>&1 || { echo "ERROR: '$1' not found in PATH" >&2; exit 1; }; }
need_root_bin git

# -------- ensure run-user exists --------
if ! id -u "$RUN_USER" >/dev/null 2>&1; then
  useradd --system --create-home --shell /usr/sbin/nologin "$RUN_USER"
fi
install -d -m0755 "$ENV_DIR" "/var/log/${APP_NAME}"

# -------- Git clone/pull under BUILD_USER with SSH agent --------
# Весь шаг выполняется в одном shell у
