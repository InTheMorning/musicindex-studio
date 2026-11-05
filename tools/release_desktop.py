#!/usr/bin/env python3
"""Make and test desktop archives on their native build runners."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import queue
import re
import secrets
import shutil
import subprocess
import tarfile
import tempfile
import threading
import tomllib
import urllib.error
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    "x86_64-pc-windows-msvc": ("Windows", "AMD64", "windows-x86_64", ".zip"),
    "x86_64-apple-darwin": ("Darwin", "x86_64", "macos-x86_64", ".tar.gz"),
    "aarch64-apple-darwin": ("Darwin", "arm64", "macos-arm64", ".tar.gz"),
    "x86_64-unknown-linux-gnu": ("Linux", "x86_64", "linux-x86_64", ".tar.gz"),
    "aarch64-unknown-linux-gnu": ("Linux", "aarch64", "linux-arm64", ".tar.gz"),
}

# Installer suffixes of each system, and the folder of the bundler for each suffix.
INSTALLERS = {
    "Windows": [".msi"],
    "Darwin": [".dmg"],
    "Linux": [".AppImage", ".deb", ".rpm"],
}
APPIMAGE_AUDIO_PLUGINS = ("autodetect", "playback", "audioconvert", "mpg123", "flac", "libav", "debugutilsbad")
BUNDLE_FOLDERS = {".AppImage": "appimage", ".deb": "deb", ".rpm": "rpm",
                  ".dmg": "dmg", ".msi": "msi"}


def version():
    with (ROOT / "Cargo.toml").open("rb") as source:
        return tomllib.load(source)["workspace"]["package"]["version"]


def target_details(target):
    if target not in TARGETS:
        raise ValueError(f"unsupported target: {target}")
    system, machine, slug, suffix = TARGETS[target]
    if (platform.system(), platform.machine().lower()) != (system, machine.lower()):
        raise ValueError(f"{target} needs a native {system} {machine} runner")
    return slug, suffix


def archive_name(target):
    slug, suffix = target_details(target)
    return f"musicindex-studio-desktop-v{version()}-{slug}{suffix}"


def package(target, binary, output):
    name = archive_name(target)
    binary = Path(binary)
    if not binary.is_file():
        raise ValueError(f"desktop binary is missing: {binary}")
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    archive = output / name
    folder = name.removesuffix(".tar.gz").removesuffix(".zip")
    files = [(binary, binary.name), (ROOT / "LICENSE", "LICENSE"),
             (ROOT / "docs/runbooks/install-desktop.md", "START-HERE.md")]
    if name.endswith(".zip"):
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
            for source, member in files:
                bundle.write(source, f"{folder}/{member}")
    else:
        with tarfile.open(archive, "w:gz") as bundle:
            for source, member in files:
                bundle.add(source, arcname=f"{folder}/{member}")
    print(archive)


def response_status(opener, url, headers=None, payload=None):
    headers = dict(headers or {})
    data = None
    if payload is not None:
        headers["Content-Type"] = "application/json"
        data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(url, data=data, headers=headers)
    try:
        with opener.open(request, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def smoke_start(binary, data, expected_vault_hash):
    process = subprocess.Popen([str(binary), "--data-dir", str(data)],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, bufsize=1)
    lines = queue.Queue()

    def read_output():
        for line in process.stdout:
            lines.put(line)
        lines.put(None)

    reader = threading.Thread(target=read_output, daemon=True)
    reader.start()
    try:
        address = None
        for _ in range(8):
            try:
                line = lines.get(timeout=20)
            except queue.Empty as error:
                raise ValueError("desktop startup timed out") from error
            if line is None:
                raise ValueError(f"desktop exited with status {process.poll()}")
            line = line.strip()
            if line.startswith("http://127.0.0.1:"):
                address = line
                break
        if address is None:
            raise ValueError("desktop did not print its local address")
        match = re.fullmatch(r"(http://127\.0\.0\.1:\d+)/#token=([A-Za-z0-9_-]+)", address)
        if match is None:
            raise ValueError("desktop printed an invalid local address")
        origin, token = match.groups()
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        status, body = response_status(opener, origin + "/")
        if status != 200 or b"Music Index Studio" not in body:
            raise ValueError("the packaged UI did not load")
        status, _ = response_status(opener, origin + "/api/v1/status")
        if status != 401:
            raise ValueError("the API accepted a request without its token")
        auth = {"Authorization": f"Bearer {token}"}
        status, body = response_status(opener, origin + "/api/v1/status", auth)
        if status != 200:
            raise ValueError("the API refused its desktop token")
        expected_state = "locked" if expected_vault_hash else "absent"
        if json.loads(body)["vault"] != expected_state:
            raise ValueError("the vault has the wrong state after startup")
        status, _ = response_status(opener, origin + "/api/v1/status",
                                    {**auth, "Host": "example.invalid"})
        if status != 403:
            raise ValueError("the API accepted a non-loopback Host")
        if not (data / "collection.sqlite").is_file():
            raise ValueError("the desktop did not create its collection")
        vault = data / "vault.json"
        if expected_vault_hash:
            if hashlib.sha256(vault.read_bytes()).digest() != expected_vault_hash:
                raise ValueError("the desktop changed the vault during restart")
            return expected_vault_hash
        status, _ = response_status(opener, origin + "/api/v1/vault", auth,
                                    {"passphrase": secrets.token_urlsafe(24)})
        if status != 201 or not vault.is_file():
            raise ValueError("the desktop did not save its vault")
        return hashlib.sha256(vault.read_bytes()).digest()
    finally:
        process.terminate()
        try:
            process.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate()


def smoke(target, archive):
    expected = archive_name(target)
    archive = Path(archive)
    if archive.name != expected:
        raise ValueError(f"expected archive {expected}")
    scratch = ROOT / ".studio-dev" / "release-smoke"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as temporary:
        folder = Path(temporary)
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as bundle:
                bundle.extractall(folder)
        else:
            with tarfile.open(archive, "r:gz") as bundle:
                bundle.extractall(folder)
        binary_name = "studio-desktop.exe" if target.endswith("windows-msvc") else "studio-desktop"
        binary = next(folder.glob(f"*/{binary_name}"))
        result = subprocess.run([str(binary), "--version"], capture_output=True, text=True,
                                timeout=10, check=True)
        if result.stdout.strip() != f"Music Index Studio {version()}":
            raise ValueError("the packaged binary has the wrong version")
        data = folder / "data"
        vault_hash = smoke_start(binary, data, None)
        smoke_start(binary, data, vault_hash)
    print(f"Smoke test passed: {expected}")


def release_parts():
    """Give the core version and the number N of an rc.N pre-release, or None."""
    match = re.fullmatch(r"(\d+\.\d+\.\d+)(?:-rc\.(\d+))?", version())
    if match is None:
        raise ValueError(f"the version {version()} is not X.Y.Z or X.Y.Z-rc.N")
    core, number = match.groups()
    if number is not None and (int(number) > 65535 or number != str(int(number))):
        raise ValueError(f"the pre-release number {number} must be 65535 or less")
    return core, number


def bundle_config(form):
    """Print the JSON for --config of the Tauri bundler (MSI and RPM refuse rc.N)."""
    core, number = release_parts()
    if form == "msi":
        wix = core if number is None else f"{core}.{number}"
        return {"bundle": {"windows": {"wix": {"version": wix}}}}
    release = "1" if number is None else f"0.rc.{number}"
    return {"version": core, "bundle": {"linux": {"rpm": {"release": release}}}}


def installer_names():
    """Give the fixed installer names of every target."""
    names = set()
    for system, _, slug, _ in TARGETS.values():
        for suffix in INSTALLERS[system]:
            names.add(f"musicindex-studio-v{version()}-{slug}{suffix}")
    return names


def check_start_bytes(path, suffix):
    size = path.stat().st_size
    with path.open("rb") as source:
        head = source.read(16)
        if suffix == ".deb":
            good = head.startswith(b"!<arch>\n")
        elif suffix == ".rpm":
            good = head.startswith(b"\xed\xab\xee\xdb")
        elif suffix == ".AppImage":
            good = head.startswith(b"\x7fELF") and head[8:11] == b"AI\x02"
        elif suffix == ".msi":
            good = head.startswith(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
        else:
            good = False
            if size >= 512:
                source.seek(size - 512)
                good = source.read(4) == b"koly"
    if not good:
        raise ValueError(f"{path.name} does not have the start bytes of a {suffix} file")


def check_appimage_audio(path):
    """Refuse an AppImage with no audio plugins, because Play then fails (ADR 0124 item 26)."""
    with tempfile.TemporaryDirectory() as folder:
        for plugin in APPIMAGE_AUDIO_PLUGINS:
            member = f"usr/lib/gstreamer-1.0/libgst{plugin}.so"
            subprocess.run([path.resolve(), "--appimage-extract", member], cwd=folder,
                           capture_output=True, check=False)
            if not (Path(folder) / "squashfs-root" / member).is_file():
                raise ValueError(
                    f"ADR 0124 item 26: {path.name} has no {member}, so Play can fail or hang the app. "
                    "Set bundleMediaFramework in tauri.conf.json, and install the "
                    "GStreamer plugin packages and patchelf in the release workflow.")


def installers(target, bundle_dir, output, skip):
    """Copy each installer of the target to its fixed name, after a start-byte check."""
    slug, _ = target_details(target)
    system = TARGETS[target][0]
    bundle_dir = Path(bundle_dir)
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    for suffix in INSTALLERS[system]:
        if suffix in skip:
            continue
        found = [path for path in (bundle_dir / BUNDLE_FOLDERS[suffix]).glob(f"*{suffix}")
                 if path.is_file()]
        if not found:
            raise ValueError(f"the installer {suffix} is missing in {bundle_dir}")
        if len(found) > 1:
            raise ValueError(f"the bundle folder has more than one {suffix} file")
        check_start_bytes(found[0], suffix)
        if suffix == ".AppImage":
            check_appimage_audio(found[0])
        destination = output / f"musicindex-studio-v{version()}-{slug}{suffix}"
        shutil.copyfile(found[0], destination)
        print(destination)


def checksums(output):
    output = Path(output)
    expected = {f"musicindex-studio-desktop-v{version()}-{slug}{suffix}"
                for _, _, slug, suffix in TARGETS.values()}
    expected |= installer_names()
    found = {path.name for path in output.iterdir() if path.is_file()}
    if found != expected:
        raise ValueError(f"file set is incomplete: {sorted(expected - found)}")
    lines = []
    for name in sorted(expected):
        digest = hashlib.sha256((output / name).read_bytes()).hexdigest()
        lines.append(f"{digest}  {name}\n")
    (output / "SHA256SUMS.txt").write_text("".join(lines), encoding="ascii")
    print(f"SHA256SUMS.txt covers all {len(expected)} desktop files")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("version")
    check.add_argument("--tag")
    pack = commands.add_parser("package")
    pack.add_argument("--target", required=True)
    pack.add_argument("--binary", required=True)
    pack.add_argument("--output", required=True)
    test = commands.add_parser("smoke")
    test.add_argument("--target", required=True)
    test.add_argument("--archive", required=True)
    config = commands.add_parser("bundle-config")
    config.add_argument("format", choices=["msi", "rpm"])
    inst = commands.add_parser("installers")
    inst.add_argument("--target", required=True)
    inst.add_argument("--bundle-dir", required=True)
    inst.add_argument("--output", required=True)
    # Only for a local check on a computer that cannot make every installer.
    inst.add_argument("--skip", action="append", default=[])
    sums = commands.add_parser("checksums")
    sums.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.command == "version":
        if args.tag and args.tag != f"v{version()}":
            raise ValueError(f"tag {args.tag} does not match v{version()}")
        print(version())
    elif args.command == "package":
        package(args.target, args.binary, args.output)
    elif args.command == "smoke":
        smoke(args.target, args.archive)
    elif args.command == "bundle-config":
        print(json.dumps(bundle_config(args.format), separators=(",", ":")))
    elif args.command == "installers":
        installers(args.target, args.bundle_dir, args.output, args.skip)
    else:
        checksums(args.output)


if __name__ == "__main__":
    main()
