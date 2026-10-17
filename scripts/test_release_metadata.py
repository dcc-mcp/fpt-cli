from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import release_metadata


class ReleaseVersionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.root_patch = patch.object(release_metadata, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.config = {
            "release": {
                "package_manifest_path": "crates/fpt-cli/Cargo.toml",
                "workspace_manifest_path": "Cargo.toml",
                "tag_prefix": "v",
            }
        }
        self.write_workspace("0.2.27")

    def write_workspace(self, version: str, lock_version: str | None = None) -> None:
        names = ("fpt-cli", "fpt-core", "fpt-domain")
        members = [f"crates/{name}" for name in names] + ["workspace-hack"]
        (self.root / "Cargo.toml").write_text(
            "[workspace]\nmembers = " + json.dumps(members)
            + f'\n[workspace.package]\nversion = "{version}"\n',
            encoding="utf-8",
        )
        for name, member in zip((*names, "workspace-hack"), members):
            path = self.root / member / "Cargo.toml"
            path.parent.mkdir(parents=True, exist_ok=True)
            package_version = '"0.1.0"' if name == "workspace-hack" else "{ workspace = true }"
            path.write_text(f'[package]\nname = "{name}"\nversion = {package_version}\n', encoding="utf-8")
        lock = "version = 4\n"
        for name in names:
            lock += f'\n[[package]]\nname = "{name}"\nversion = "{lock_version or version}"\n'
        lock += '\n[[package]]\nname = "workspace-hack"\nversion = "0.1.0"\n'
        # A registry package with the same name must not be treated as a local member.
        lock += '\n[[package]]\nname = "fpt-core"\nversion = "9.8.7"\nsource = "registry+https://example.invalid/index"\nchecksum = "unchanged"\n'
        (self.root / "Cargo.lock").write_text(lock, encoding="utf-8")
        (self.root / ".release-please-manifest.json").write_text(json.dumps({".": version}), encoding="utf-8")

    def test_release_tag_rejects_stale_local_lock_versions(self) -> None:
        self.write_workspace("0.2.27", lock_version="0.2.26")
        with self.assertRaisesRegex(SystemExit, r"Cargo.lock.*fpt-cli"):
            release_metadata.verify_tag(self.config, "v0.2.27")

    def test_accepts_inherited_and_independent_member_versions_without_writing(self) -> None:
        before = (self.root / "Cargo.lock").read_bytes()
        release_metadata.verify_workspace_versions(self.config)
        release_metadata.verify_tag(self.config, "v0.2.27")
        self.assertEqual((self.root / "Cargo.lock").read_bytes(), before)

    def test_accepts_a_later_release_without_hard_coded_versions(self) -> None:
        self.write_workspace("3.4.5")
        release_metadata.verify_workspace_versions(self.config)
        release_metadata.verify_tag(self.config, "v3.4.5")

    def test_rejects_missing_local_member_even_with_a_registry_namesake(self) -> None:
        path = self.root / "Cargo.lock"
        path.write_text(path.read_text().replace('\n[[package]]\nname = "fpt-core"\nversion = "0.2.27"\n', ""), encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, r"Cargo.lock.*fpt-core"):
            release_metadata.verify_workspace_versions(self.config)

    def test_rejects_duplicate_local_member(self) -> None:
        path = self.root / "Cargo.lock"
        path.write_text(path.read_text() + '\n[[package]]\nname = "fpt-cli"\nversion = "0.2.27"\n', encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, r"Cargo.lock.*fpt-cli"):
            release_metadata.verify_workspace_versions(self.config)

    def test_rejects_release_manifest_drift(self) -> None:
        (self.root / ".release-please-manifest.json").write_text('{".": "0.2.26"}', encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, r"release-please-manifest"):
            release_metadata.verify_workspace_versions(self.config)


if __name__ == "__main__":
    unittest.main()
