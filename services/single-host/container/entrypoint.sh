#!/usr/bin/env bash
set -euo pipefail

PROFILE_PATH=${NOA_PROFILE:-/etc/noa/server/profiles/single_host/profile.toml}
SYSTEMD_UNIT_PATH=${SYSTEMD_UNIT_PATH:-/etc/systemd/system}

mkdir -p "${SYSTEMD_UNIT_PATH}/noa"
cp -f /opt/noa/services/single-host/systemd/noa-single-host.target "${SYSTEMD_UNIT_PATH}/noa-single-host.target"
cp -f /opt/noa/services/single-host/systemd/noa-single-host@.service "${SYSTEMD_UNIT_PATH}/noa-single-host@.service"

export NOA_PROFILE="${PROFILE_PATH}"

if [[ ${NOA_SKIP_SYSTEMD:-0} -eq 1 ]]; then
    echo "[entrypoint] Systemd disabled; launching services directly"
    /opt/noa/services/single-host/init/noa-single-host.sh start all
else
    echo "[entrypoint] Enabling noa-single-host.target"
    systemctl daemon-reload
    systemctl enable noa-single-host.target
    systemctl start noa-single-host.target
fi

exec "$@"
