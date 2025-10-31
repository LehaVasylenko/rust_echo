#!/usr/bin/env bash
set -euo pipefail

APP_NAME="${1:-myrustapp}"
REPO_URL="${2:-git@github.com:LehaVasylenko/rust_echo.git}"
BRANCH="${3:-main}"
RUN_USER="${4:-appuser}"

APP_DIR="/opt/${APP_NAME}"
ENV_DIR="/etc/${APP_NAME}"
ENV_FILE="${ENV_DIR}/env"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"
SRC_DIR="/tmp/${APP_NAME}-src"

need() { command -v "$1" >/dev/null 2>&1 || { echo "ERROR: '$1' not found in PATH"; exit 1; }; }
need git
need cargo

# 1) user + dirs
if ! id -u "$RUN_USER" >/dev/null 2>&1; then
  useradd --system --create-home --shell /usr/sbin/nologin "$RUN_USER"
fi
install -d -m0755 "$APP_DIR" "$ENV_DIR" "/var/log/${APP_NAME}"

# 2) clone/pull
if [[ -d "$SRC_DIR/.git" ]]; then
  git -C "$SRC_DIR" fetch --all --quiet
  git -C "$SRC_DIR" checkout "$BRANCH"
  git -C "$SRC_DIR" reset --hard "origin/${BRANCH}"
else
  rm -rf "$SRC_DIR"
  git clone --branch "$BRANCH" --depth 1 "$REPO_URL" "$SRC_DIR"
fi

# 3) figure out binary name from Cargo.toml (package.name) as default
BIN_NAME="$(awk -F'= *' '/^\[package\]/{f=1} f && /^name *=/ {gsub(/"/,"",$2); print $2; exit}' "$SRC_DIR/Cargo.toml" || true)"
if [[ -z "$BIN_NAME" ]]; then
  echo "WARN: can't parse package.name; will auto-pick newest executable from target/release"
fi

# 4) build (prefer locked if Cargo.lock exists)
pushd "$SRC_DIR" >/dev/null
if [[ -f Cargo.lock ]]; then
  cargo build --release --locked
else
  cargo build --release
fi
popd >/dev/null

# 5) locate built binary
BIN_PATH=""
if [[ -n "${BIN_NAME}" && -f "$SRC_DIR/target/release/${BIN_NAME}" ]]; then
  BIN_PATH="$SRC_DIR/target/release/${BIN_NAME}"
else
  # fallback: pick newest executable in target/release (skip .d, .rlib, .so, .a)
  BIN_PATH="$(find "$SRC_DIR/target/release" -maxdepth 1 -type f -perm -111 \
    ! -name "*.d" ! -name "*.rlib" ! -name "*.so" ! -name "*.a" 2>/dev/null \
    -printf "%T@ %p\n" | sort -nr | awk 'NR==1{print $2}')"
fi

if [[ -z "$BIN_PATH" || ! -f "$BIN_PATH" ]]; then
  echo "ERROR: built binary not found in target/release"; exit 1
fi
echo "[*] using binary: $BIN_PATH"

# 6) deploy binary
install -m0755 "$BIN_PATH" "${APP_DIR}/${APP_NAME}"
chown -R "$RUN_USER":"$RUN_USER" "$APP_DIR" "/var/log/${APP_NAME}"

# 7) env file (create/update, keep your knobs here)
cat >"$ENV_FILE" <<EOF
# RUST_LOG controls logging (env_filter), e.g. debug,info,trace
RUST_LOG=${RUST_LOG:-info}

# APP_OPTS are passed to your binary as-is (flags, config paths, etc.)
APP_OPTS=${APP_OPTS:-}
EOF
chmod 0644 "$ENV_FILE"

# 8) systemd unit
cat >"$SERVICE_FILE" <<EOF
[Unit]
Description=${APP_NAME} (Rust service)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${RUN_USER}
WorkingDirectory=${APP_DIR}
EnvironmentFile=${ENV_FILE}
ExecStart=${APP_DIR}/${APP_NAME} \$APP_OPTS
Restart=always
RestartSec=2
# Tweak limits if needed:
LimitNOFILE=65536

# Pass RUST_LOG to the process environment
Environment=RUST_LOG=\$RUST_LOG

# Journald logging (default). To redirect, uncomment:
# StandardOutput=journal
# StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# 9) reload + enable + start
systemctl daemon-reload
systemctl enable --now "${APP_NAME}.service"

echo "[OK] ${APP_NAME} deployed & started via systemd"
echo "Status:   systemctl status ${APP_NAME}"
echo "Logs:     journalctl -u ${APP_NAME} -f"
echo "Config:   ${ENV_FILE}  (edit & systemctl restart ${APP_NAME})"
