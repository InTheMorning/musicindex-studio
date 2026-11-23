#!/bin/sh
# Start the test SFTP server and Studio with the data of WORK, and drive the
# UI with one steps file. Both stop when the drive ends. See README.md.
#
# Usage: tools/tutorial-walk/run.sh WORK OUT STEPS_JSON
#
# STUDIO_BIN selects the Studio binary. The default is the
# .studio-dev/v-*/studio-desktop with the newest change time. run.sh writes its
# version. Make sure that it is the version of the tutorial. On new data,
# run.sh sets the library to WORK/library, except when NOLIB=1. OUT and
# STEPS_JSON are absolute paths.
set -eu
WORK=${1:?usage: run.sh WORK OUT STEPS_JSON}
OUT=${2:?usage: run.sh WORK OUT STEPS_JSON}
STEPS=${3:?usage: run.sh WORK OUT STEPS_JSON}
for p in "$WORK" "$OUT" "$STEPS"; do
  case "$p" in /*) ;; *) echo "run.sh: $p is not an absolute path" >&2; exit 2 ;; esac
done
cd "$(dirname "$0")/../.."
BIN=${STUDIO_BIN:-$(ls -t "$PWD"/.studio-dev/v-*/studio-desktop | head -1)}
STUDIO_PORT=${STUDIO_PORT:-47293}
mkdir -p "$OUT"
export HOME="$WORK/h" XDG_CONFIG_HOME="$WORK/c" XDG_CACHE_HOME="$WORK/x"
rm -f "$WORK/data/studio.lock"

echo "run: sshd"
/usr/bin/sshd -D -f "$WORK/sshk/cfg" -E "$WORK/sshk/log" &
SSHD=$!
NEW=0; [ -f "$WORK/data/collection.sqlite" ] || NEW=1
LOG="$OUT/server.log"
echo "run: studio $BIN"
echo "run: version $("$BIN" --version 2>/dev/null | head -1)"
"$BIN" --data-dir "$WORK/data" --port "$STUDIO_PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV $SSHD 2>/dev/null; true' EXIT
timeout 60 sh -c "until grep -q token= '$LOG'; do sleep 0.3; done"
URL=$(grep -o 'http://[^ ]*#token=[0-9a-f]*' "$LOG" | head -1)
TOKEN=${URL##*token=}
if [ "$NEW" = 1 ] && [ -z "${NOLIB:-}" ]; then
  echo "run: library"
  curl -s -X PUT -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
    -d "{\"root\":\"$WORK/library\"}" "http://127.0.0.1:$STUDIO_PORT/api/v1/library"; echo
fi
echo "run: drive $STEPS"
DRIVE_UI_PORT=${DRIVE_UI_PORT:-9563} timeout 600 tools/node tools/drive-ui.mjs "$URL" "$OUT" "$(cat "$STEPS")"
echo "run: done"
