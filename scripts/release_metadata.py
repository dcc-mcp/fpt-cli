from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
CONFIG_PATH = ROOT / "distribution.toml"


def load_distribution() -> dict[str, Any]:
    return tomllib.loads(CONFIG_PATH.read_text(encoding="utf-8"))


def load_toml(path: str) -> dict[str, Any]:
    return tomllib.loads((ROOT / path).read_text(encoding="utf-8"))


def release_version(config: dict[str, Any]) -> str:
    release = config["release"]
    package_manifest = load_toml(release["package_manifest_path"])
    package_version = package_manifest["package"]["version"]

    if isinstance(package_version, str):
        return package_version

    if isinstance(package_version, dict) and package_version.get("workspace") is True:
        workspace_manifest = load_toml(release["workspace_manifest_path"])
        return workspace_manifest["workspace"]["package"]["version"]

    raise SystemExit("unsupported package.version format in package manifest")


def release_tag(config: dict[str, Any]) -> str:
    return f"{config['release']['tag_prefix']}{release_version(config)}"


def verify_workspace_versions(config: dict[str, Any]) -> None:
    workspace_path = Path(config["release"]["workspace_manifest_path"])
    workspace = load_toml(str(workspace_path))["workspace"]
    workspace_version = workspace["package"]["version"]
    lock = load_toml(str(workspace_path.parent / "Cargo.lock"))
    local_packages = [package for package in lock["package"] if "source" not in package]

    for member in workspace["members"]:
        paths = sorted((ROOT / workspace_path.parent).glob(member))
        if not paths:
            raise SystemExit(f"workspace member {member} has no package manifest")
        for path in paths:
            package = tomllib.loads((path / "Cargo.toml").read_text(encoding="utf-8"))["package"]
            name = package["name"]
            version = package["version"]
            if isinstance(version, dict) and version.get("workspace") is True:
                version = workspace_version
            matches = [entry for entry in local_packages if entry["name"] == name]
            if len(matches) != 1:
                raise SystemExit(f"Cargo.lock must contain one local {name} package, found {len(matches)}")
            if matches[0]["version"] != version:
                raise SystemExit(
                    f"Cargo.lock local package {name} version {matches[0]['version']} "
                    f"does not match manifest version {version}"
                )

    manifest = json.loads((ROOT / ".release-please-manifest.json").read_text(encoding="utf-8"))
    if manifest["."] != release_version(config):
        raise SystemExit(".release-please-manifest.json version does not match the release package")


def build_matrix(config: dict[str, Any]) -> dict[str, Any]:
    include = []
    for target in config["targets"]:
        include.append(
            {
                "os": target["os"],
                "target": target["target"],
                "archive_name": target["archive_name"],
                "binary_name": target["binary_name"],
            }
        )
    return {"include": include}


def verify_tag(config: dict[str, Any], tag_name: str) -> None:
    verify_workspace_versions(config)
    expected = release_tag(config)
    if tag_name != expected:
        raise SystemExit(
            f"tag {tag_name} does not match release version {release_version(config)} (expected {expected})"
        )


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print("usage: release_metadata.py <version|tag|matrix|verify-workspace|verify-tag> [tag]", file=sys.stderr)
        return 1

    config = load_distribution()
    command = argv[1]

    if command == "version":
        print(release_version(config))
        return 0

    if command == "tag":
        print(release_tag(config))
        return 0

    if command == "matrix":
        print(json.dumps(build_matrix(config), separators=(",", ":")))
        return 0

    if command == "verify-workspace":
        verify_workspace_versions(config)
        print("verified workspace, lockfile, and release manifest versions")
        return 0

    if command == "verify-tag":
        if len(argv) != 3:
            print("usage: release_metadata.py verify-tag <tag>", file=sys.stderr)
            return 1
        verify_tag(config, argv[2])
        print(f"verified {argv[2]}")
        return 0

    print(f"unknown command: {command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
