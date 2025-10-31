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
  start_ssh_agent_if_needed
  # первый коннект к github/gitlab: авто-принятие хоста
  host="$(echo "$REPO_URL" | sed -E 's|^[^@]+@([^/:]+).*|\1|')"
  ssh -o StrictHostKeyChecking=accept-new -T "$host" >/dev/null 2>&1 || true
else
  SSH_WAS_STARTED=0
fi

### 2) git clone/fetch (под текущим пользователем)
if [[ -d "$SRC_DIR/.git" ]]; then
  echo "[git] fetch/reset $BRANCH in $SRC_DIR"
  git -C "$SRC_DIR" fetch --all --quiet
  git -C "$SRC_DIR" checkout "$BRANCH"
  git -C "$SRC_DIR" reset --hard "origin/${BRANCH}"
else
  echo "[git] clone $REPO_URL -> $SRC_DIR (branch $BRANCH)"
  rm -rf "$SRC_DIR"
  git clone --branch "$BRANCH" --depth 1 "$REPO_URL" "$SRC_DIR"
fi

### 3) build (под текущим пользователем)
echo "[build] cargo build --release"
export PATH="$HOME_DIR/.cargo/bin:$PATH"
pushd "$SRC_DIR" >/dev/null
if [[ -f Cargo.lock ]]; then
  "$CARGO_BIN" build --release --locked
else
  "$CARGO_BIN" build --release
fi
popd >/dev/null

### 4) найти имя пакета и бинарь
BIN_NAME="$(awk -F'= *' '/^\[package\]/{f=1} f && /^name *=/ {gsub(/"/,"",$2); print $2; exit}' "$SRC_DIR/Cargo.toml" || true)"
BIN_PATH=""
if [[ -n "$BIN_NAME" && -f "$SRC_DIR/target/release/$BIN_NAME" ]]; then
  BIN_PATH="$SRC_DIR/target/release/$BIN_NAME"
else
  BIN_PATH="$(find "$SRC_DIR/target/release" -maxdepth 1 -type f -perm -111 \
    ! -name '*.d' ! -name '*.rlib' ! -name '*.so' ! -name '*.a' 2>/dev/null \
    -printf '%T@ %p\n' | sort -nr | head -n1 | awk '{print $2}')"
fi
[[ -n "$BIN_PATH" && -f "$BIN_PATH" ]] || { echo "ERROR: built binary not found"; exit 1; }
echo "[build] binary: $BIN_PATH"

### 5) deploy + systemd (требует sudo)
echo "[sudo] устанавливаю файлы и systemd-юнит"
sudo install -d -m0755 "$APP_DIR" "$ENV_DIR"
sudo install -m0755 "$BIN_PATH" "${APP_DIR}/${APP_NAME}"

# env-файл: правь тут RUST_LOG/APP_OPTS по вкусу
sudo tee "$ENV_FILE" >/dev/null <<EOF
RUST_LOG=${RUST_LOG:-info}
APP_OPTS=${APP_OPTS:-}
EOF
sudo chmod 0644 "$ENV_FILE"

# unit-файл: сервис будет работать ПОД ТЕКУЩИМ пользователем
sudo tee "$SERVICE_FILE" >/dev/null <<EOF
[Unit]
Description=${APP_NAME} (Rust)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${USER_NAME}
WorkingDirectory=${APP_DIR}
EnvironmentFile=${ENV_FILE}
Environment=RUST_LOG=\$RUST_LOG
ExecStart=${APP_DIR}/${APP_NAME} \$APP_OPTS
Restart=always
RestartSec=2
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now "${APP_NAME}.service"

echo "[OK] ${APP_NAME} deployed & started as systemd service (User=${USER_NAME})"
echo "Status:   sudo systemctl status ${APP_NAME}"
echo "Logs:     journalctl -u ${APP_NAME} -f"
echo "Config:   ${ENV_FILE}  (edit & sudo systemctl restart ${APP_NAME})"

### 6) аккуратно погасим временный ssh-agent, если мы его запускали
if [[ "${SSH_WAS_STARTED:-0}" -eq 1 && -n "${SSH_AGENT_PID:-}" ]]; then
  echo "[ssh] stopping ssh-agent"
  ssh-agent -k >/dev/null 2>&1 || true
fi
