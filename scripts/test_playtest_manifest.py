#!/usr/bin/env python3
"""Regression checks for platform package source/content provenance.

Fixtures live entirely in temporary directories. Git metadata is stubbed;
these checks neither invoke a build nor inspect or change a developer's Git tree.
"""

import importlib.util
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = __file__.replace("\\", "/").rsplit("/", 1)[0] + "/playtest-manifest.py"
SPEC = importlib.util.spec_from_file_location("playtest_manifest", SCRIPT)
manifest = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(manifest)


class PlaytestManifestTests(unittest.TestCase):
    COMMIT = "1" * 40

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="rebellion-manifest-test-")
        self.addCleanup(temporary.cleanup)
        self.root = manifest.Path(temporary.name)
        self.fixture_files = {
            "src/gameplay.rs": b"const DAMAGE: u32 = 10;\n",
            "src/platform/input.rs": b"// Shared controller policy.\n",
            "config/balance.json": b'{"enemy_health":100}\n',
            "games/elder_fleet/config/module.json": b'{"missions":9}\n',
            "Cargo.toml": b'[package]\nname = "fixture"\n',
            "Cargo.lock": b"version = 4\n",
            ".cargo/config.toml": b"[build]\njobs = 2\n",
            "assets/ships/587.png": b"fixture-ship-image",
            "assets/audio/shot.wav": b"fixture-shot-audio",
        }
        for name, data in self.fixture_files.items():
            self.write(name, data)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def make(self, target="aarch64-apple-darwin", commit=None, status=""):
        with patch.object(
            manifest.subprocess,
            "check_output",
            side_effect=[(commit or self.COMMIT) + "\n", status],
        ):
            return manifest.make_manifest(self.root, target)

    def copy_assets(self):
        package = self.root / "package" / "assets"
        for name, data in self.fixture_files.items():
            if name.startswith("assets/"):
                path = package / name.removeprefix("assets/")
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
        return package

    def manifest_file(self, value):
        path = self.root / "PLAYTEST-MANIFEST.json"
        path.write_text(manifest.json.dumps(value), encoding="utf-8")
        return path

    def test_exact_packaged_assets_are_accepted(self):
        manifest.verify_assets(self.make(), self.copy_assets())

    def test_packaged_missing_stale_and_extra_assets_are_rejected(self):
        expected = self.make()
        for mutation, diagnostic in (
            ("missing", "missing=['ships/587.png']"),
            ("stale", "changed=['ships/587.png']"),
            ("extra", "extra=['ships/extra.png']"),
        ):
            with self.subTest(mutation=mutation):
                package = self.copy_assets()
                ship = package / "ships/587.png"
                extra = package / "ships/extra.png"
                if extra.exists():
                    extra.unlink()
                if mutation == "missing":
                    ship.unlink()
                elif mutation == "stale":
                    ship.write_bytes(b"outdated-image")
                else:
                    extra.write_bytes(b"unlisted-image")
                with self.assertRaises(ValueError) as error:
                    manifest.verify_assets(expected, package)
                self.assertIn(diagnostic, str(error.exception))

    def test_missing_packaged_asset_directory_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Missing input directory"):
            manifest.verify_assets(self.make(), self.root / "absent-assets")

    def test_platform_target_metadata_preserves_shared_fingerprints(self):
        targets = (
            "aarch64-apple-darwin",
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "wasm32-unknown-unknown",
        )
        packages = [self.make(target) for target in targets]
        self.assertEqual([package["target"] for package in packages], list(targets))
        for field in ("shared_source_sha256", "assets_sha256"):
            self.assertEqual(len({package[field] for package in packages}), 1)
        manifest.compare_manifests(packages, require_clean=True)

    def test_gameplay_and_both_configuration_sources_change_parity(self):
        original = self.make()
        for name in (
            "src/gameplay.rs",
            "config/balance.json",
            "games/elder_fleet/config/module.json",
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
        ):
            with self.subTest(changed_input=name):
                self.write(name, self.fixture_files[name] + b"changed\n")
                changed = self.make("wasm32-unknown-unknown")
                self.assertNotEqual(
                    original["shared_source_sha256"], changed["shared_source_sha256"]
                )
                self.assertEqual(original["assets_sha256"], changed["assets_sha256"])
                with self.assertRaisesRegex(ValueError, "shared_source_sha256"):
                    manifest.compare_manifests([original, changed])
                self.write(name, self.fixture_files[name])

    def test_changed_asset_source_fails_platform_comparison(self):
        original = self.make()
        self.write("assets/audio/shot.wav", b"different-audio")
        changed = self.make("wasm32-unknown-unknown")
        with self.assertRaisesRegex(ValueError, "assets_sha256"):
            manifest.compare_manifests([original, changed])

    def test_require_clean_rejects_dirty_first_or_subsequent_package(self):
        clean = self.make()
        for status in (" M src/gameplay.rs\n", "?? new-source.rs\n"):
            modified = self.make("wasm32-unknown-unknown", status=status)
            self.assertEqual(modified["working_tree"], "modified")
            for packages in ([clean, modified], [modified, clean]):
                with self.subTest(status=status, first=packages[0]["target"]):
                    with self.assertRaisesRegex(ValueError, "one clean source commit"):
                        manifest.compare_manifests(packages, require_clean=True)

    def test_require_clean_rejects_otherwise_equal_different_commits(self):
        original = self.make()
        different_commit = self.make("wasm32-unknown-unknown", commit="2" * 40)
        # Local source-equivalence checks need not claim common CI provenance.
        manifest.compare_manifests([original, different_commit])
        with self.assertRaisesRegex(ValueError, "one clean source commit"):
            manifest.compare_manifests([original, different_commit], require_clean=True)

    def test_read_manifest_accepts_unmodified_fingerprints(self):
        expected = self.make()
        self.assertEqual(manifest.read_manifest(self.manifest_file(expected)), expected)

    def test_tampered_file_inventories_are_rejected(self):
        for field in ("shared_source_files", "asset_files"):
            with self.subTest(field=field):
                tampered = self.make()
                first_file = next(iter(tampered[field]))
                tampered[field][first_file] = "0" * 64
                with self.assertRaisesRegex(ValueError, f"Invalid {field} fingerprint"):
                    manifest.read_manifest(self.manifest_file(tampered))

    def test_tampered_summary_digests_are_rejected(self):
        for field in ("shared_source_sha256", "assets_sha256"):
            with self.subTest(field=field):
                tampered = self.make()
                tampered[field] = "0" * 64
                with self.assertRaisesRegex(ValueError, "Invalid .* fingerprint"):
                    manifest.read_manifest(self.manifest_file(tampered))

    def test_missing_or_empty_inventory_is_rejected(self):
        for field in ("shared_source_files", "asset_files"):
            for remove in (False, True):
                with self.subTest(field=field, removed=remove):
                    tampered = self.make()
                    if remove:
                        del tampered[field]
                    else:
                        tampered[field] = {}
                    with self.assertRaisesRegex(ValueError, f"Invalid {field} fingerprint"):
                        manifest.read_manifest(self.manifest_file(tampered))

    def test_unsupported_manifest_schema_is_rejected(self):
        tampered = self.make()
        tampered["schema_version"] = manifest.SCHEMA + 1
        with self.assertRaisesRegex(ValueError, "Unsupported manifest schema"):
            manifest.read_manifest(self.manifest_file(tampered))

    def test_required_provenance_metadata_is_validated(self):
        invalid = {
            "target": (None, "", "  ", 123),
            "source_commit": (None, "", "1" * 39, "1" * 41, "z" * 40, 123),
            "working_tree": (None, "", "unknown", True, ["clean"]),
        }
        for field, values in invalid.items():
            for value in values:
                with self.subTest(field=field, value=value):
                    tampered = self.make()
                    if value is None:
                        del tampered[field]
                    else:
                        tampered[field] = value
                    with self.assertRaisesRegex(ValueError, f"Invalid {field} metadata"):
                        manifest.read_manifest(self.manifest_file(tampered))

    def test_non_object_manifest_is_rejected(self):
        for value in (["manifest"], "manifest", None):
            with self.subTest(value=value):
                with self.assertRaisesRegex(ValueError, "Invalid manifest object"):
                    manifest.read_manifest(self.manifest_file(value))

    def test_non_mapping_inventory_is_rejected_even_with_matching_hash(self):
        for field, digest in (
            ("shared_source_files", "shared_source_sha256"),
            ("asset_files", "assets_sha256"),
        ):
            for value in (["invented-file"], "invented-file", 123):
                with self.subTest(field=field, value=value):
                    tampered = self.make()
                    tampered[field] = value
                    tampered[digest] = manifest.tree_hash(value)
                    with self.assertRaisesRegex(ValueError, f"Invalid {field} fingerprint"):
                        manifest.read_manifest(self.manifest_file(tampered))

    def test_inventory_paths_must_be_normalized_relative_posix_names(self):
        for name in ("", "/ship.png", "a//b", "a/../b", "a/./b", "a/",
                     "C:/ship.png", "ships\\587.png"):
            with self.subTest(name=name):
                tampered = self.make()
                tampered["asset_files"] = {name: "a" * 64}
                tampered["assets_sha256"] = manifest.tree_hash(tampered["asset_files"])
                with self.assertRaisesRegex(ValueError, "relative POSIX path"):
                    manifest.read_manifest(self.manifest_file(tampered))

    def test_inventory_file_digests_must_be_lowercase_sha256(self):
        for digest in (None, 123, "a" * 63, "a" * 65, "A" * 64, "z" * 64):
            with self.subTest(digest=digest):
                tampered = self.make()
                tampered["shared_source_files"] = {"src/gameplay.rs": digest}
                tampered["shared_source_sha256"] = manifest.tree_hash(
                    tampered["shared_source_files"]
                )
                with self.assertRaisesRegex(ValueError, "file digest"):
                    manifest.read_manifest(self.manifest_file(tampered))


if __name__ == "__main__":
    unittest.main()
