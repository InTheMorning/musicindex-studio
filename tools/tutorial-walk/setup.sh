#!/bin/sh
# Make a work folder for a tutorial walk: a test SFTP server with key login,
# test media, and empty Studio data. See tools/tutorial-walk/README.md.
#
# Usage: tools/tutorial-walk/setup.sh WORK [SSH_PORT]
#
# WORK must be an absolute path outside the repository. Put the test feeds in
# WORK/host after this step. The SFTP user is the current user.
set -eu
WORK=${1:?usage: setup.sh WORK [SSH_PORT]}
PORT=${2:-2299}
case "$WORK" in /*) ;; *) echo "WORK must be an absolute path" >&2; exit 2 ;; esac
HERE=$(cd "$(dirname "$0")" && pwd)

mkdir -p "$WORK/sshk" "$WORK/host" "$WORK/media" "$WORK/data" "$WORK/library" "$WORK/h" "$WORK/c" "$WORK/x"
echo "setup: ssh keys"
[ -f "$WORK/sshk/hk" ] || ssh-keygen -q -t ed25519 -N '' -f "$WORK/sshk/hk"
[ -f "$WORK/sshk/user" ] || ssh-keygen -q -t ed25519 -N '' -f "$WORK/sshk/user"
cp "$WORK/sshk/user.pub" "$WORK/sshk/authorized_keys"
cat > "$WORK/sshk/cfg" <<EOF
Port $PORT
ListenAddress 127.0.0.1
HostKey $WORK/sshk/hk
PidFile $WORK/sshk/pid
UsePAM no
StrictModes no
AuthorizedKeysFile $WORK/sshk/authorized_keys
PasswordAuthentication no
Subsystem sftp internal-sftp
EOF
echo "$PORT" > "$WORK/sshk/port"
echo "setup: test media"
python3 -I "$HERE/media.py" "$WORK/media"
echo "setup: done. Put the test feeds in $WORK/host"
