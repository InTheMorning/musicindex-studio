#!/usr/bin/env bash
#
# Deploy a release candidate of studio-server to the VPS (ADR 0066).
# The VPS pulls the image from GHCR. This script builds nothing.
#
# Usage:
#   tools/deploy-server.sh --release 0.1.0-rc.1 [--host USER@HOST] [--dir PATH]
#
# The repository holds no host and no folder of the VPS. The script reads
# STUDIO_DEPLOY_HOST and STUDIO_DEPLOY_DIR from the environment, or from the
# file .studio-dev/deploy.env. Git ignores that file.
#
set -euo pipefail

LOCAL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE_PREFIX="ghcr.io/inthemorning/studio-server"

# A value in the environment has priority over the file.
VPS="${STUDIO_DEPLOY_HOST:-}"
VPS_DIR="${STUDIO_DEPLOY_DIR:-}"
if [[ -f "${LOCAL_DIR}/.studio-dev/deploy.env" ]]; then
    # shellcheck source=/dev/null
    source "${LOCAL_DIR}/.studio-dev/deploy.env"
    VPS="${VPS:-${STUDIO_DEPLOY_HOST:-}}"
    VPS_DIR="${VPS_DIR:-${STUDIO_DEPLOY_DIR:-}}"
fi

usage() {
    cat >&2 <<'USAGE'
usage: deploy-server.sh --release VERSION [--host USER@HOST] [--dir PATH]

  --release VERSION
              Deploy the GHCR image of VERSION, for example 0.1.0-rc.1.
              VERSION must have a hyphen (ADR 0066 item 9). The local tag
              vVERSION must exist. The script builds nothing.
  --host USER@HOST
              The VPS. The default is STUDIO_DEPLOY_HOST.
  --dir PATH  The folder of compose.yml on the VPS. The default is
              STUDIO_DEPLOY_DIR.

The script reads STUDIO_DEPLOY_HOST and STUDIO_DEPLOY_DIR from the
environment, or from .studio-dev/deploy.env. Git ignores that file.

The script copies deploy/compose.yml to the folder. It does not copy .env.
The folder on the VPS must have a .env file (see deploy/env.example).
The VPS pulls the image first, and nothing stops when the pull fails.
Then the script pins the image in .env and runs docker compose up -d.
Last, it waits at most 60 seconds for two facts:
  - The revision label of the container is the commit of the tag vVERSION.
  - The log of the container has the line "Music Index Studio server VERSION".
The script makes no account and no invite code.
USAGE
}

release=""

while [[ $# -gt 0 ]]; do
    arg="$1"
    shift
    case "${arg}" in
        --release)
            if [[ $# -eq 0 ]]; then
                usage
                exit 1
            fi
            release="$1"
            shift
            ;;
        --host)
            if [[ $# -eq 0 ]]; then
                usage
                exit 1
            fi
            VPS="$1"
            shift
            ;;
        --dir)
            if [[ $# -eq 0 ]]; then
                usage
                exit 1
            fi
            VPS_DIR="$1"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            usage
            exit 1
            ;;
    esac
done

if [[ -z "${release}" ]]; then
    usage
    exit 1
fi
if [[ -z "${VPS}" || -z "${VPS_DIR}" ]]; then
    echo >&2 "error: set STUDIO_DEPLOY_HOST and STUDIO_DEPLOY_DIR, or use --host and --dir."
    echo >&2 "       .studio-dev/deploy.env can hold the two values."
    exit 1
fi

# --- Check the version and its local tag. No connection before this ends. ---

if [[ ! "${release}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-[0-9A-Za-z.-]+$ ]]; then
    echo >&2 "error: --release needs a version with a hyphen, such as 0.1.0-rc.1, not ${release}"
    exit 1
fi
if ! expected_revision="$(git -C "${LOCAL_DIR}" rev-list -n 1 "v${release}" 2>/dev/null)"; then
    echo >&2 "error: the local tag v${release} does not exist. Tag the release first."
    exit 1
fi
image="${IMAGE_PREFIX}:${release}"
echo ":: Release ${release}, commit ${expected_revision}"

# --- Copy the compose file. Do not copy .env. ---

echo ":: Copying deploy/compose.yml to ${VPS}:${VPS_DIR}"
ssh "${VPS}" "mkdir -p $(printf %q "${VPS_DIR}")"
rsync -z "${LOCAL_DIR}/deploy/compose.yml" "${VPS}:${VPS_DIR}/compose.yml"

# --- On the VPS: pull, pin, start and check ---

echo ":: Deploying on the VPS"
ssh "${VPS}" \
    "VPS_DIR=$(printf %q "${VPS_DIR}") IMAGE=$(printf %q "${image}") RELEASE=$(printf %q "${release}") EXPECTED_REVISION=$(printf %q "${expected_revision}") bash -s" <<'REMOTE'
set -euo pipefail
cd "${VPS_DIR}"

if [[ ! -f .env ]]; then
    echo >&2 "error: ${VPS_DIR}/.env does not exist on the VPS."
    echo >&2 "       Make it from deploy/env.example, then run this script again."
    exit 1
fi

# Pull first. When the pull fails, nothing has changed.
echo "Pulling ${IMAGE}..."
docker pull "${IMAGE}"

# Pin the image in .env, so that a plain `docker compose up -d` keeps it.
if grep -q '^STUDIO_SERVER_IMAGE=' .env; then
    sed -i "s#^STUDIO_SERVER_IMAGE=.*#STUDIO_SERVER_IMAGE=${IMAGE}#" .env
else
    echo "STUDIO_SERVER_IMAGE=${IMAGE}" >> .env
fi
echo "Pinned in .env: STUDIO_SERVER_IMAGE=${IMAGE}"

docker compose up -d

# Wait at most 60 seconds. The start takes several seconds (Argon2).
expected_line="Music Index Studio server ${RELEASE}"
actual_revision=""
line_found=0
for _ in $(seq 1 30); do
    id="$(docker compose ps -q studio | head -n1)"
    if [[ -n "${id}" ]]; then
        actual_revision="$(docker inspect -f '{{ index .Config.Labels "org.opencontainers.image.revision" }}' "${id}" 2>/dev/null || true)"
        # Read the whole log first. With pipefail, `docker logs | grep -q`
        # can fail when grep stops at the first match.
        log="$(docker logs "${id}" 2>&1 || true)"
        if grep -qxF "${expected_line}" <<<"${log}"; then
            line_found=1
        else
            line_found=0
        fi
        if [[ "${actual_revision}" == "${EXPECTED_REVISION}" && "${line_found}" -eq 1 ]]; then
            break
        fi
    fi
    sleep 2
done

if [[ "${actual_revision}" != "${EXPECTED_REVISION}" ]]; then
    echo >&2 "error: the revision label of the container is '${actual_revision}', and v${RELEASE} is '${EXPECTED_REVISION}'"
    exit 1
fi
if [[ "${line_found}" -ne 1 ]]; then
    echo >&2 "error: the log of the container has no line '${expected_line}'"
    echo >&2 "       Expected: ${expected_line}"
    echo >&2 "       Found (first log line): $(docker logs "${id}" 2>&1 | head -n1)"
    exit 1
fi
echo "   revision ${actual_revision} is the commit of v${RELEASE}"
echo "   log line found: ${expected_line}"
REMOTE

echo ":: Done"
