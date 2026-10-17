from __future__ import annotations

import io
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OLD_BINARY = b"previous installation\n"
NEW_BINARY = b"fixture release\n"
VERSION = "v9.8.7"
REPOSITORY = "fixture-owner/fpt-cli"
BASE_URL = f"https://github.com/{REPOSITORY}/releases"

CURL_FIXTURE = r"""#!/bin/sh
output=
head=false
url=
while [ "$#" -gt 0 ]; do
    case "$1" in
        -o|--output) output="$2"; shift ;;
        -w|--write-out) shift ;;
        *I*) case "$1" in -*) head=true ;; *) url="$1" ;; esac ;;
        -*) ;;
        *) url="$1" ;;
    esac
    shift
done
printf '%s\n' "$url" >> "$FPT_TEST_REQUESTS"
if [ "$head" = true ]; then
    [ "$url" = "$FPT_TEST_BASE/latest" ] || exit 91
    [ "$FPT_TEST_MODE" != lookup_failure ] || { echo 'fixture latest lookup failed' >&2; exit 22; }
    printf '%s' "$FPT_TEST_LATEST_URL"
else
    [ "$url" = "$FPT_TEST_ASSET_URL" ] || { echo "unexpected fixture URL: $url" >&2; exit 92; }
    [ "$FPT_TEST_MODE" != asset_failure ] || { echo 'fixture asset download failed' >&2; exit 22; }
    cp "$FPT_TEST_ARCHIVE" "$output"
fi
"""

POWERSHELL_FIXTURE = r"""
param([string]$Version)
$ErrorActionPreference = 'Stop'
function Invoke-RestMethod {
    param([string]$Uri)
    Add-Content -LiteralPath $env:FPT_TEST_REQUESTS -Value $Uri
    if ($Uri -ne "https://api.github.com/repos/$env:FPT_INSTALL_REPOSITORY/releases/latest") { throw "Unexpected fixture metadata URL: $Uri" }
    if ($env:FPT_TEST_MODE -eq 'lookup_failure') { throw 'fixture latest lookup failed' }
    return (Get-Content -LiteralPath $env:FPT_TEST_METADATA -Raw | ConvertFrom-Json)
}
function Invoke-WebRequest {
    param([string]$Uri, [string]$OutFile)
    Add-Content -LiteralPath $env:FPT_TEST_REQUESTS -Value $Uri
    if ($Uri -ne $env:FPT_TEST_ASSET_URL) { throw "Unexpected fixture asset URL: $Uri" }
    if ($env:FPT_TEST_MODE -eq 'asset_failure') { throw 'fixture asset download failed' }
    Copy-Item -LiteralPath $env:FPT_TEST_ARCHIVE -Destination $OutFile
}
if ($PSBoundParameters.ContainsKey('Version')) {
    & $env:FPT_TEST_INSTALLER -Version $Version
} else {
    & $env:FPT_TEST_INSTALLER
}
"""


def posix_shell() -> str | None:
    if os.name == "nt":
        candidate = Path(os.environ.get("ProgramFiles", "C:/Program Files")) / "Git/bin/sh.exe"
        return str(candidate) if candidate.is_file() else None
    return shutil.which("sh")


class InstallerFixtures(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory(prefix="fpt-installer-fixture-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.install = self.root / "install with spaces"
        self.install.mkdir()
        self.temp = self.root / "temporary files"
        self.temp.mkdir()
        self.requests = self.root / "requests.log"
        self.env = os.environ.copy()
        self.env.update({
            "FPT_INSTALL_REPOSITORY": REPOSITORY,
            "FPT_INSTALL_DIR": str(self.install),
            "FPT_TEST_REQUESTS": str(self.requests),
            "FPT_TEST_MODE": "success",
            "TEMP": str(self.temp),
            "TMP": str(self.temp),
            "TMPDIR": str(self.temp),
        })
        self.env.pop("FPT_INSTALL_VERSION", None)

    def calls(self) -> list[str]:
        return self.requests.read_text(encoding="utf-8-sig").splitlines() if self.requests.exists() else []

    def run_script(self, command: list[str], binary: str) -> subprocess.CompletedProcess[str]:
        (self.install / binary).write_bytes(OLD_BINARY)
        return subprocess.run(
            command, env=self.env, capture_output=True, text=True, timeout=30,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )

    def assert_result(self, result: subprocess.CompletedProcess[str], binary: str, success: bool) -> None:
        detail = result.stdout + result.stderr
        self.assertEqual(result.returncode == 0, success, detail)
        self.assertEqual((self.install / binary).read_bytes(), NEW_BINARY if success else OLD_BINARY, detail)
        self.assertEqual(list(self.temp.iterdir()), [], detail)

    def prepare_posix(self, os_name: str = "Linux", arch: str = "x86_64") -> list[str]:
        shell = posix_shell()
        if shell is None:
            self.skipTest("a POSIX shell is not available")
        target = {("Linux", "x86_64"): "x86_64-unknown-linux-gnu", ("Darwin", "x86_64"): "x86_64-apple-darwin", ("Darwin", "arm64"): "aarch64-apple-darwin"}[(os_name, arch)]
        archive = self.root / "release.tar.gz"
        with tarfile.open(archive, "w:gz") as package:
            entry = tarfile.TarInfo("fpt")
            entry.size = len(NEW_BINARY)
            entry.mode = 0o755
            package.addfile(entry, io.BytesIO(NEW_BINARY))
        tools = self.root / "tools"
        tools.mkdir(exist_ok=True)
        (tools / "curl").write_text(CURL_FIXTURE, encoding="utf-8", newline="\n")
        (tools / "uname").write_text(f'#!/bin/sh\ncase "$1" in -s) echo {os_name};; -m) echo {arch};; *) exit 93;; esac\n', encoding="utf-8", newline="\n")
        for path in tools.iterdir():
            path.chmod(0o755)
        self.env.update({
            "PATH": str(tools) + os.pathsep + self.env["PATH"],
            "FPT_TEST_BASE": BASE_URL,
            "FPT_TEST_LATEST_URL": f"{BASE_URL}/tag/{VERSION}",
            "FPT_TEST_ASSET_URL": f"{BASE_URL}/download/{VERSION}/fpt-{VERSION}-{target}.tar.gz",
            "FPT_TEST_ARCHIVE": archive.as_posix(),
            "FPT_INSTALL_DIR": self.install.as_posix(),
            "FPT_TEST_REQUESTS": self.requests.as_posix(),
            "FPT_TEST_TOOLS": tools.as_posix(),
        })
        wrapper = 'tools="$FPT_TEST_TOOLS"; '
        if os.name == "nt":
            wrapper += 'tools="$(cygpath -u "$tools")"; '
        wrapper += 'PATH="$tools:$PATH"; export PATH; [ "$(command -v curl)" = "$tools/curl" ] || exit 94; exec sh "$@"'
        return [shell, "-c", wrapper, "fixture", str(ROOT / "scripts/install.sh")]

    def prepare_powershell(self, shell: str) -> list[str]:
        archive = self.root / "release.zip"
        with zipfile.ZipFile(archive, "w") as package:
            package.writestr("fpt.exe", NEW_BINARY)
        metadata = self.root / "metadata.json"
        metadata.write_text(json.dumps({"tag_name": VERSION}), encoding="utf-8")
        wrapper = self.root / "fixture.ps1"
        wrapper.write_text(POWERSHELL_FIXTURE, encoding="utf-8")
        self.env.update({
            "FPT_TEST_INSTALLER": str(ROOT / "scripts/install.ps1"),
            "FPT_TEST_METADATA": str(metadata),
            "FPT_TEST_ASSET_URL": f"{BASE_URL}/download/{VERSION}/fpt-{VERSION}-x86_64-pc-windows-msvc.zip",
            "FPT_TEST_ARCHIVE": str(archive),
        })
        return [shell, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", str(wrapper)]

    def test_posix_latest_uses_versioned_archive_for_each_target(self) -> None:
        for os_name, arch in [("Linux", "x86_64"), ("Darwin", "x86_64"), ("Darwin", "arm64")]:
            with self.subTest(os=os_name, arch=arch):
                self.requests.unlink(missing_ok=True)
                command = self.prepare_posix(os_name, arch)
                self.assert_result(self.run_script(command, "fpt"), "fpt", True)
                self.assertEqual(self.calls(), [f"{BASE_URL}/latest", self.env["FPT_TEST_ASSET_URL"]])

    def test_posix_explicit_version_skips_latest(self) -> None:
        for version in [VERSION, VERSION[1:]]:
            with self.subTest(version=version):
                self.requests.unlink(missing_ok=True)
                command = self.prepare_posix()
                self.assert_result(self.run_script(command + [version], "fpt"), "fpt", True)
                self.assertEqual(self.calls(), [self.env["FPT_TEST_ASSET_URL"]])

    def test_posix_environment_version_skips_latest(self) -> None:
        command = self.prepare_posix()
        self.env["FPT_INSTALL_VERSION"] = VERSION
        self.assert_result(self.run_script(command, "fpt"), "fpt", True)
        self.assertEqual(self.calls(), [self.env["FPT_TEST_ASSET_URL"]])

    def test_posix_failures_preserve_existing_binary(self) -> None:
        for mode in ["lookup_failure", "asset_failure", "invalid_latest", "corrupt_archive"]:
            with self.subTest(mode=mode):
                self.requests.unlink(missing_ok=True)
                command = self.prepare_posix()
                self.env["FPT_TEST_MODE"] = mode
                if mode == "invalid_latest":
                    self.env["FPT_TEST_LATEST_URL"] = "https://example.invalid/releases/tag/v9.8.7"
                if mode == "corrupt_archive":
                    Path(self.env["FPT_TEST_ARCHIVE"]).write_bytes(b"not an archive")
                result = self.run_script(command, "fpt")
                self.assert_result(result, "fpt", False)
                self.assertTrue(result.stderr.strip(), result.stdout)
                expected = [f"{BASE_URL}/latest"]
                if mode not in {"lookup_failure", "invalid_latest"}:
                    expected.append(self.env["FPT_TEST_ASSET_URL"])
                self.assertEqual(self.calls(), expected)

    @unittest.skipUnless(os.name == "nt", "Windows PowerShell installer requires Windows")
    def test_powershell_real_installer_latest_explicit_and_failures(self) -> None:
        shells = [path for name in ["powershell", "pwsh"] if (path := shutil.which(name))]
        self.assertTrue(shells, "Windows must execute a real PowerShell installer")
        for shell in shells:
            for mode in ["latest", "explicit", "environment", "lookup_failure", "asset_failure", "invalid_latest", "corrupt_archive"]:
                with self.subTest(shell=Path(shell).name, mode=mode):
                    self.requests.unlink(missing_ok=True)
                    command = self.prepare_powershell(shell)
                    self.env.pop("FPT_INSTALL_VERSION", None)
                    self.env["FPT_TEST_MODE"] = mode
                    if mode == "explicit":
                        command += ["-Version", VERSION[1:]]
                    elif mode == "environment":
                        self.env["FPT_INSTALL_VERSION"] = VERSION
                    elif mode == "invalid_latest":
                        Path(self.env["FPT_TEST_METADATA"]).write_text('{"tag_name":"../../bad"}', encoding="utf-8")
                    elif mode == "corrupt_archive":
                        Path(self.env["FPT_TEST_ARCHIVE"]).write_bytes(b"not an archive")
                    success = mode in {"latest", "explicit", "environment"}
                    result = self.run_script(command, "fpt.exe")
                    self.assert_result(result, "fpt.exe", success)
                    metadata_url = f"https://api.github.com/repos/{REPOSITORY}/releases/latest"
                    expected = [] if mode in {"explicit", "environment"} else [metadata_url]
                    if mode not in {"lookup_failure", "invalid_latest"}:
                        expected.append(self.env["FPT_TEST_ASSET_URL"])
                    self.assertEqual(self.calls(), expected)
                    if not success:
                        self.assertTrue(result.stderr.strip(), result.stdout)


if __name__ == "__main__":
    unittest.main()
