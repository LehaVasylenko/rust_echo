#!/usr/bin/env bash
set -euo pipefail

### === НАСТРОЙКИ ПОД СЕБЯ (минимум) ===
APP_NAME="rust-echo"                                   # имя сервиса/бинаря
REPO_URL="git@github.com:LehaVasylenko/rust_echo.git"  # git ssh/https URL
BRANCH="main"                                          # ветка

### === КОНСТАНТЫ ===
USER_NAME="$(id -un)"
HOME_DIR="$HOME"
CARGO_BIN="${HOME_DIR}/.cargo/bin/cargo"
SRC_DIR="/tmp/${APP_NAME}-src"
APP_DIR="/opt/${APP_NAME}"
ENV_DIR="/etc/${APP_NAME}"
ENV_FILE="${ENV_DIR}/env"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"

echo "[run] app=${APP_NAME} repo=${REPO_URL} branch=${BRANCH} user=${USER_NAME}"

### 0) проверим cargo
if [[ ! -x "$CARGO_BIN" ]]; then
  echo "ERROR: cargo not found at $CARGO_BIN"
  echo "Установи Rust:  curl https://sh.rustup.rs | sh  &&  source ~/.cargo/env"
  exit 1
fi

### 1) ssh-agent при необходимости (только для ssh URL)
start_ssh_agent_if_needed() {
  if ! ssh-add -l >/dev/null 2>&1; then
    echo "[ssh] starting ssh-agent..."
    eval "$(ssh-agent -s)"
    # авто-добавление стандартных ключей (если есть). Без ошибок, если их нет.
    [[ -f "$HOME_DIR/.ssh/id_ed25519" ]] && ssh-add "$HOME_DIR/.ssh/id_ed25519" >/dev/null 2>&1 || true
    [[ -f "$HOME_DIR/.ssh/id_rsa"     ]] && ssh-add "$HOME_DIR/.ssh/id_rsa"     >/dev/null 2>&1 || true
    SSH_WAS_STARTED=1
  else
    echo "[ssh] agent already running"
    SSH_WAS_STARTED=0
  fi
}
if [[ "$REPO_URL" =~ ^git@ || "$REPO_URL" =~ ^ssh:// ]]; then
  sta
