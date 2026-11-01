#!/usr/bin/env bash
#
# Make a release candidate of Studio and deploy it (ADR 0066).
#
# Usage:
#   tools/release-candidate.sh VERSION [--no-deploy] [--yes]
#
# The script does the steps of the runbooks in their sequence:
#   1. It sets VERSION in Cargo.toml and Cargo.lock, and commits only these
#      two files. When the version is already VERSION, it makes no commit.
#   2. It pushes main, and waits for CI on that commit (ADR 0066 item 7).
#   3. It pushes the tag vVERSION. The tag starts server-image.yml and
#      desktop-release.yml.
#   4. It waits for the server image of the tag.
#   5. It runs tools/deploy-server.sh --release VERSION.
#
# A second run with the same VERSION continues after the last completed
# step. The script does not publish the desktop draft.
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

usage() {
    cat >&2 <<'USAGE'
usage: release-candidate.sh VERSION [--no-deploy] [--yes]

  VERSION      The candidate version, for example 0.1.0-rc.8. It must have
               a hyphen (ADR 0066 item 9).
  --no-deploy  Stop after the server image. Do not run deploy-server.sh.
  --yes        Do not ask before the first push.

The script waits for GitHub Actions through the GitHub API. It uses
`gh api` when gh has a login. Otherwise it uses curl, with GITHUB_TOKEN
when it is set. With no token, it asks one time each minute, because
GitHub permits 60 requests in each hour with no token.
USAGE
}

die() {
    echo "release-candidate: $*" >&2
    exit 1
}

version=""
deploy=1
confirm=1
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-deploy) deploy=0 ;;
        --yes) confirm=0 ;;
        -h | --help)
            usage
            exit 0
            ;;
        -*)
            usage
            exit 1
            ;;
        *)
            [[ -z "${version}" ]] || { usage; exit 1; }
            version="$1"
            ;;
    esac
    shift
done
[[ -n "${version}" ]] || { usage; exit 1; }
[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-[0-9A-Za-z.]+$ ]] \
    || die "ADR 0066 item 9: VERSION must be a candidate with a hyphen, for example 0.1.0-rc.8."
tag="v${version}"

repo="$(git remote get-url origin | sed -E 's#^.*github\.com[:/]##; s#\.git$##')"
[[ "${repo}" == */* ]] || die "origin is not a GitHub repository."

if gh auth status >/dev/null 2>&1; then
    use_gh=1
    interval=30
else
    use_gh=0
    interval=$([[ -n "${GITHUB_TOKEN:-}" ]] && echo 30 || echo 60)
fi

# One GET request to the GitHub API. It prints the JSON body.
api() {
    if [[ "${use_gh}" == 1 ]]; then
        gh api "$1"
    else
        curl -fsSL -H 'Accept: application/vnd.github+json' \
            ${GITHUB_TOKEN:+-H "Authorization: Bearer ${GITHUB_TOKEN}"} \
            "https://api.github.com/$1"
    fi
}

# The status, the conclusion and the address of the newest push run of
# workflow $1 on commit $2.
run_state() {
    api "repos/${repo}/actions/workflows/$1/runs?head_sha=$2&event=push&per_page=1" | python3 -c '
import json, sys
runs = json.load(sys.stdin).get("workflow_runs") or [{}]
run = runs[0]
print(run.get("status") or "none", run.get("conclusion") or "none", run.get("html_url") or "-")'
}

# Wait for workflow $1 on commit $2 for at most $3 minutes. Stop when the run
# does not pass.
wait_for() {
    local workflow="$1" sha="$2" minutes="$3" last="" status conclusion url
    local end=$((SECONDS + minutes * 60))
    while ((SECONDS < end)); do
        read -r status conclusion url < <(run_state "${workflow}" "${sha}")
        if [[ "${status}" != "${last}" ]]; then
            echo "  ${workflow} on ${sha:0:7}: ${status} ${url}"
            last="${status}"
        fi
        if [[ "${status}" == completed ]]; then
            [[ "${conclusion}" == success ]] && return 0
            die "${workflow} on ${sha:0:7} ended with ${conclusion}: ${url}"
        fi
        sleep "${interval}"
    done
    die "${workflow} on ${sha:0:7} did not end within ${minutes} minutes."
}

current="$(python3 tools/release_desktop.py version)"
git fetch --quiet origin main
local_tag="$(git rev-parse -q --verify "refs/tags/${tag}^{commit}" || true)"
remote_tag="$(git ls-remote --tags origin "refs/tags/${tag}" | cut -f1)"
if [[ -n "${local_tag}" && -n "${remote_tag}" && "${local_tag}" != "${remote_tag}" ]]; then
    die "the local tag ${tag} and the tag on origin are on different commits. Use a new version."
fi
tagged="${local_tag:-${remote_tag}}"

# The plan. A tag that exists fixes the commit, and the script skips the
# version commit and the push of main.
steps=()
if [[ -n "${tagged}" ]]; then
    git cat-file -e "${tagged}^{commit}" 2>/dev/null || git fetch --quiet origin "refs/tags/${tag}:refs/tags/${tag}"
    [[ "$(git show "${tagged}:Cargo.toml" | sed -nE 's/^version = "(.*)"$/\1/p')" == "${version}" ]] \
        || die "the tag ${tag} exists, but Cargo.toml of its commit has a different version. Use a new version."
    git merge-base --is-ancestor "${tagged}" origin/main \
        || die "the tag ${tag} is on a commit that is not on origin/main. Push main first."
    sha="${tagged}"
else
    [[ "$(git symbolic-ref --short HEAD)" == main ]] || die "check out main first."
    git merge-base --is-ancestor origin/main HEAD \
        || die "origin/main has commits that main does not have. Pull first."
    count="$(git rev-list --count origin/main..HEAD)"
    if [[ "${current}" != "${version}" ]]; then
        git diff --quiet HEAD -- Cargo.toml Cargo.lock \
            || die "Cargo.toml or Cargo.lock has changes that are not committed."
        steps+=("Set the version ${current} -> ${version} in Cargo.toml and Cargo.lock, and commit them.")
        count=$((count + 1))
    fi
    steps+=("Push main (${count} new commits) and wait for CI.")
    steps+=("Push the tag ${tag}.")
fi
[[ -z "${remote_tag}" && -n "${local_tag}" ]] && steps+=("Push the tag ${tag}.")
steps+=("Wait for server-image.yml on the tag.")
[[ "${deploy}" == 1 ]] && steps+=("Run tools/deploy-server.sh --release ${version}.")

echo "Release candidate ${version} of ${repo}:"
printf '  - %s\n' "${steps[@]}"
if [[ "${confirm}" == 1 ]]; then
    read -r -p "Continue? [y/N] " answer
    [[ "${answer}" == [yY]* ]] || die "stopped. Nothing was changed."
fi

if [[ -z "${tagged}" ]]; then
    if [[ "${current}" != "${version}" ]]; then
        echo "Set the version to ${version}."
        sed -i -E "s/^version = \"${current//./\\.}\"$/version = \"${version}\"/" Cargo.toml
        [[ "$(python3 tools/release_desktop.py version)" == "${version}" ]] \
            || die "Cargo.toml does not give ${version} after the change. Examine Cargo.toml."
        cargo update --workspace --offline --quiet
        git add Cargo.toml Cargo.lock
        git commit --quiet -m "Set the version to ${version}" \
            -m "The tag ${tag} builds the desktop drafts and the image ghcr.io/inthemorning/studio-server:${version}."
    fi
    sha="$(git rev-parse HEAD)"
    echo "Push main."
    git push --quiet origin main
    echo "Wait for CI (ADR 0066 item 7)."
    wait_for ci.yml "${sha}" 45
    git tag "${tag}" "${sha}"
fi

if [[ -z "${remote_tag}" ]]; then
    echo "Push the tag ${tag}."
    git push --quiet origin "refs/tags/${tag}"
fi
git rev-parse -q --verify "refs/tags/${tag}" >/dev/null \
    || git fetch --quiet origin "refs/tags/${tag}:refs/tags/${tag}"

echo "Wait for the server image."
wait_for server-image.yml "${sha}" 90

if [[ "${deploy}" == 1 ]]; then
    echo "Deploy ${version}."
    tools/deploy-server.sh --release "${version}"
fi

read -r status conclusion url < <(run_state desktop-release.yml "${sha}")
echo "Done. The desktop draft workflow is ${status} ${conclusion}: ${url}"
echo "The draft release stays a draft (docs/runbooks/desktop-release.md)."
