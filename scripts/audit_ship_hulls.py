#!/usr/bin/env python3
"""
Rebellion ship hull audit.

Checks:
- Manifest shape and uniqueness
- Every declared sprite exists and is a readable RGBA PNG
- Every numeric PNG in assets/ships is declared
- Approved SHA-256 and dHash locks
- Exact and near-duplicate sprite images
- Transparent background and visible-content bounds
- Hull IDs referenced by source/config files
- Name/class mismatches in Rust tuple match arms
- Faction mapping mismatches in get_enemy_color/get_faction_engine_trail
- Optional contact sheet for visual approval

Dependencies:
    python 3.10+
    pip install pillow jsonschema

Pillow is required for image checks/contact sheets.
jsonschema is optional; core validation still runs without it.

Typical use:
    python scripts/audit_ship_hulls.py \
      --root . \
      --manifest assets/ships/ship_manifest.json \
      --schema assets/ships/ship_manifest.schema.json \
      --report build/ship-audit.md \
      --json-report build/ship-audit.json \
      --contact-sheet build/ship-contact-sheet.png \
      --strict

Hash approval workflow:
    1. Review the contact sheet.
    2. Correct sprite/orientation/centering.
    3. Run with --print-locks.
    4. Copy SHA-256/dHash values into approved manifest entries.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Iterable

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    Image = None
    ImageDraw = None
    ImageFont = None

try:
    import jsonschema
except ImportError:
    jsonschema = None


SOURCE_SUFFIXES = {".rs", ".json", ".toml", ".ron", ".yaml", ".yml"}
SCAN_DIRS = ("src", "config", "games")
EXCLUDED_DIRS = {".git", "target", "build", "dist", "web/pkg", "node_modules"}

CLASS_MAP = {
    "Frigate": "frigate",
    "Destroyer": "destroyer",
    "Cruiser": "cruiser",
    "Battlecruiser": "battlecruiser",
    "Battleship": "battleship",
}

FACTION_TOKEN_MAP = {
    "COLOR_MINMATAR": "minmatar",
    "COLOR_AMARR": "amarr",
    "COLOR_CALDARI": "caldari",
    "COLOR_GALLENTE": "gallente",
    "COLOR_TRIGLAVIAN": "triglavian",
    "EngineTrail::minmatar": "minmatar",
    "EngineTrail::amarr": "amarr",
    "EngineTrail::caldari": "caldari",
    "EngineTrail::gallente": "gallente",
    "EngineTrail::triglavian": "triglavian",
    "EngineTrail::edencom": "edencom",
    "EngineTrail::pirate": "pirate",
}


@dataclass
class Finding:
    severity: str
    code: str
    message: str
    path: str | None = None
    line: int | None = None
    type_id: int | None = None


def add(findings: list[Finding], severity: str, code: str, message: str,
        path: Path | str | None = None, line: int | None = None,
        type_id: int | None = None) -> None:
    findings.append(Finding(
        severity=severity,
        code=code,
        message=message,
        path=str(path) if path is not None else None,
        line=line,
        type_id=type_id,
    ))


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def dhash_image(image: Any) -> str:
    gray = image.convert("L").resize((9, 8))
    pixels = list(gray.getdata())
    value = 0
    for y in range(8):
        for x in range(8):
            left = pixels[y * 9 + x]
            right = pixels[y * 9 + x + 1]
            value = (value << 1) | int(left > right)
    return f"{value:016x}"


def hamming_hex(a: str, b: str) -> int:
    return (int(a, 16) ^ int(b, 16)).bit_count()


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def brace_body(text: str, function_name: str) -> str | None:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\b", text)
    if not match:
        return None
    start = text.find("{", match.end())
    if start < 0:
        return None
    depth = 0
    for index in range(start, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[start + 1:index]
    return None


def expand_id_expression(expr: str) -> set[int]:
    ids: set[int] = set()
    for start, end in re.findall(r"\b(\d{2,8})\s*\.\.=\s*(\d{2,8})\b", expr):
        a, b = int(start), int(end)
        if 0 <= b - a <= 100:
            ids.update(range(a, b + 1))
    expr = re.sub(r"\b\d{2,8}\s*\.\.=\s*\d{2,8}\b", " ", expr)
    ids.update(int(value) for value in re.findall(r"\b\d{2,8}\b", expr))
    return ids


def iter_match_arms(body: str) -> Iterable[tuple[str, str]]:
    # Good enough for Rebellion's lookup tables: each arm begins on a new line.
    pattern = re.compile(
        r"(?ms)^\s*([^/\n][^=\n]*?(?:\n\s*[^=\n]+?)?)\s*=>\s*(.*?)"
        r"(?=^\s*(?:[0-9_][^=\n]*|_)\s*=>|\Z)"
    )
    for match in pattern.finditer(body):
        yield match.group(1).strip(), match.group(2).strip()


def source_files(root: Path) -> Iterable[Path]:
    for directory in SCAN_DIRS:
        base = root / directory
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in SOURCE_SUFFIXES:
                continue
            rel = path.relative_to(root).as_posix()
            if any(rel == item or rel.startswith(item + "/") for item in EXCLUDED_DIRS):
                continue
            yield path


def collect_source_references(root: Path) -> tuple[dict[int, list[tuple[str, int, str]]], list[tuple[int, str, str, int]]]:
    references: dict[int, list[tuple[str, int, str]]] = {}
    named_arms: list[tuple[int, str, str, int]] = []

    explicit_patterns = [
        re.compile(r"\b(?:type_id|ship_type_id|spawn_type_id)\s*:\s*(\d{2,8})\b"),
        re.compile(r"\b(?:type_id|ship_type_id|spawn_type_id)\s*=\s*(\d{2,8})\b"),
        re.compile(r"\b(?:from_type_id|get)\s*\(\s*(\d{2,8})\s*\)"),
    ]
    named_arm = re.compile(
        r"^\s*(\d{2,8})\s*=>\s*\(\s*\"([^\"]+)\".*?ShipClass::([A-Za-z_]+)",
        re.MULTILINE
    )

    for path in source_files(root):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        rel = path.relative_to(root).as_posix()
        lines = text.splitlines()

        for pattern in explicit_patterns:
            for match in pattern.finditer(text):
                type_id = int(match.group(1))
                line = text.count("\n", 0, match.start()) + 1
                context = lines[line - 1].strip() if line <= len(lines) else ""
                references.setdefault(type_id, []).append((rel, line, context))

        # Preload arrays and faction/class/rotation lookup lines are ship-ID-dense.
        if any(token in rel for token in (
            "ship_sprites.rs", "enemy/faction.rs", "enemy/spawn.rs",
            "player.rs", "boss.rs", "wingman.rs", "ship_select.rs"
        )):
            for line_number, line_text in enumerate(lines, 1):
                if any(token in line_text for token in (
                    "ShipClass::", "EngineTrail::", "WeaponType::", "COLOR_",
                    "rotation", "sprite", "carrier", "frigate", "destroyer",
                    "cruiser", "battleship", "titan"
                )):
                    # In spawn.rs, avoid false positives from stat tuples like
                    # 597 => ("Punisher", 40.0, 80.0, 100, ShipClass::Frigate).
                    # Only capture the first number before => or the first number on the line.
                    if "spawn.rs" in rel and "=>" in line_text:
                        match = re.search(r"^\s*(\d{3,8})\s*=>", line_text)
                        if match:
                            token = match.group(1)
                            type_id = int(token)
                            references.setdefault(type_id, []).append(
                                (rel, line_number, line_text.strip())
                            )
                        continue
                    for token in re.findall(r"\b\d{3,8}\b", line_text):
                        type_id = int(token)
                        references.setdefault(type_id, []).append(
                            (rel, line_number, line_text.strip())
                        )

        for match in named_arm.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            named_arms.append((int(match.group(1)), match.group(2), match.group(3), line))

    return references, named_arms


def inspect_mapping_function(
    root: Path,
    path: Path,
    function_name: str,
    manifest_by_id: dict[int, dict[str, Any]],
    findings: list[Finding],
) -> None:
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    body = brace_body(text, function_name)
    if body is None:
        return

    for left, right in iter_match_arms(body):
        ids = expand_id_expression(left)
        inferred = None
        for token, faction in FACTION_TOKEN_MAP.items():
            if token in right:
                inferred = faction
                break
        if inferred is None:
            continue

        for type_id in ids:
            ship = manifest_by_id.get(type_id)
            if not ship:
                continue
            expected = ship["faction"]
            # pirate is intentionally broader than a named pirate lineage.
            compatible = inferred == expected or (
                inferred == "pirate" and expected in {"guristas", "pirate"}
            )
            if not compatible:
                add(
                    findings,
                    "error",
                    "FACTION_MAPPING_MISMATCH",
                    f"{function_name} maps {ship['name']} ({type_id}) to "
                    f"{inferred}, but the manifest says {expected}.",
                    path.relative_to(root),
                    type_id=type_id,
                )


def build_contact_sheet(
    root: Path,
    ships: list[dict[str, Any]],
    output: Path,
    findings: list[Finding],
) -> None:
    if Image is None:
        add(findings, "warning", "PILLOW_MISSING",
            "Pillow is not installed; contact sheet was not generated.")
        return

    thumb = 180
    label_h = 64
    columns = 5
    rows = (len(ships) + columns - 1) // columns
    sheet = Image.new("RGBA", (columns * thumb, rows * (thumb + label_h)), (12, 16, 24, 255))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.load_default()

    for index, ship in enumerate(sorted(ships, key=lambda item: item["type_id"])):
        col = index % columns
        row = index // columns
        x = col * thumb
        y = row * (thumb + label_h)
        path = root / ship["sprite"]["path"]

        draw.rectangle((x, y, x + thumb - 1, y + thumb - 1), outline=(70, 90, 120, 255))
        if path.exists():
            try:
                image = Image.open(path).convert("RGBA")
                image.thumbnail((thumb - 20, thumb - 20))
                px = x + (thumb - image.width) // 2
                py = y + (thumb - image.height) // 2
                sheet.alpha_composite(image, (px, py))
            except Exception:
                draw.text((x + 10, y + 10), "INVALID IMAGE", fill=(255, 80, 80, 255), font=font)
        else:
            draw.text((x + 10, y + 10), "MISSING", fill=(255, 80, 80, 255), font=font)

        labels = [
            f"{ship['type_id']}  {ship['name']}",
            f"{ship['faction']} / {ship['class']}",
            ship["status"],
        ]
        for offset, label in enumerate(labels):
            draw.text((x + 5, y + thumb + 4 + offset * 17), label[:28],
                      fill=(225, 232, 242, 255), font=font)

    output.parent.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(output)


def markdown_report(findings: list[Finding], stats: dict[str, Any]) -> str:
    counts = {
        level: sum(1 for finding in findings if finding.severity == level)
        for level in ("error", "warning", "info")
    }
    lines = [
        "# Rebellion Ship Hull Audit",
        "",
        f"- Manifest hulls: **{stats['manifest_hulls']}**",
        f"- Numeric PNG assets: **{stats['asset_pngs']}**",
        f"- Source hull references: **{stats['source_reference_ids']}**",
        f"- Errors: **{counts['error']}**",
        f"- Warnings: **{counts['warning']}**",
        f"- Informational: **{counts['info']}**",
        "",
    ]
    for severity in ("error", "warning", "info"):
        group = [finding for finding in findings if finding.severity == severity]
        if not group:
            continue
        lines.extend([f"## {severity.title()}s", ""])
        for finding in group:
            location = ""
            if finding.path:
                location = f" — `{finding.path}"
                if finding.line:
                    location += f":{finding.line}"
                location += "`"
            hull = f" [type {finding.type_id}]" if finding.type_id is not None else ""
            lines.append(f"- **{finding.code}**{hull}: {finding.message}{location}")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--manifest", type=Path, default=Path("assets/ships/ship_manifest.json"))
    parser.add_argument("--schema", type=Path, default=Path("assets/ships/ship_manifest.schema.json"))
    parser.add_argument("--report", type=Path, default=Path("build/ship-audit.md"))
    parser.add_argument("--json-report", type=Path, default=Path("build/ship-audit.json"))
    parser.add_argument("--contact-sheet", type=Path, default=None)
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("--print-locks", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else root / args.manifest
    schema_path = args.schema if args.schema.is_absolute() else root / args.schema
    report_path = args.report if args.report.is_absolute() else root / args.report
    json_report_path = (
        args.json_report if args.json_report.is_absolute() else root / args.json_report
    )
    contact_path = None
    if args.contact_sheet:
        contact_path = (
            args.contact_sheet if args.contact_sheet.is_absolute()
            else root / args.contact_sheet
        )

    findings: list[Finding] = []

    if not manifest_path.exists():
        add(findings, "error", "MANIFEST_MISSING", f"Manifest not found: {manifest_path}")
        print(markdown_report(findings, {
            "manifest_hulls": 0, "asset_pngs": 0, "source_reference_ids": 0
        }))
        return 2

    manifest = load_json(manifest_path)
    ships = manifest.get("ships", [])

    if schema_path.exists() and jsonschema is not None:
        try:
            jsonschema.validate(instance=manifest, schema=load_json(schema_path))
        except Exception as exc:
            add(findings, "error", "SCHEMA_VALIDATION", str(exc))
    elif jsonschema is None:
        add(findings, "warning", "JSONSCHEMA_MISSING",
            "jsonschema is not installed; only built-in checks were run.")
    else:
        add(findings, "warning", "SCHEMA_MISSING", f"Schema not found: {schema_path}")

    manifest_by_id: dict[int, dict[str, Any]] = {}
    seen_slugs: set[str] = set()
    seen_paths: set[str] = set()

    for ship in ships:
        type_id = ship.get("type_id")
        slug = ship.get("slug")
        sprite_path = ship.get("sprite", {}).get("path")

        if not isinstance(type_id, int):
            add(findings, "error", "TYPE_ID_INVALID", f"Invalid type_id in entry: {ship!r}")
            continue
        if type_id in manifest_by_id:
            add(findings, "error", "DUPLICATE_TYPE_ID",
                f"Duplicate manifest type ID {type_id}.", type_id=type_id)
        manifest_by_id[type_id] = ship

        if slug in seen_slugs:
            add(findings, "error", "DUPLICATE_SLUG", f"Duplicate slug {slug!r}.", type_id=type_id)
        seen_slugs.add(slug)

        if sprite_path in seen_paths:
            add(findings, "error", "DUPLICATE_SPRITE_PATH",
                f"Multiple hulls use {sprite_path}.", type_id=type_id)
        seen_paths.add(sprite_path)

        canonical_path = f"assets/ships/{type_id}.png"
        if sprite_path != canonical_path:
            add(findings, "warning", "NONCANONICAL_SPRITE_PATH",
                f"Expected {canonical_path}, found {sprite_path}.", type_id=type_id)

    asset_dir = root / manifest.get("asset_root", "assets/ships")
    asset_pngs: dict[int, Path] = {}
    if asset_dir.exists():
        for path in asset_dir.glob("*.png"):
            if path.stem.isdigit():
                asset_pngs[int(path.stem)] = path
    else:
        add(findings, "error", "ASSET_DIR_MISSING", f"Asset directory not found: {asset_dir}")

    exact_hashes: dict[str, list[int]] = {}
    dhashes: dict[int, str] = {}
    locks: dict[int, dict[str, str]] = {}

    for type_id, ship in sorted(manifest_by_id.items()):
        rel_path = Path(ship["sprite"]["path"])
        path = root / rel_path
        if not path.exists():
            add(findings, "error", "SPRITE_MISSING",
                f"{ship['name']} has no sprite at {rel_path}.", rel_path, type_id=type_id)
            continue

        digest = sha256_file(path)
        exact_hashes.setdefault(digest, []).append(type_id)
        expected_hash = ship["sprite"].get("approved_sha256")
        if expected_hash and digest.lower() != expected_hash.lower():
            add(findings, "error", "APPROVED_HASH_CHANGED",
                f"{ship['name']} sprite differs from its approved SHA-256.",
                rel_path, type_id=type_id)

        if Image is None:
            continue

        try:
            image = Image.open(path)
            image.load()
        except Exception as exc:
            add(findings, "error", "IMAGE_UNREADABLE",
                f"{ship['name']} could not be decoded: {exc}", rel_path, type_id=type_id)
            continue

        if image.format != "PNG":
            add(findings, "error", "IMAGE_NOT_PNG",
                f"{ship['name']} reports format {image.format}, expected PNG.",
                rel_path, type_id=type_id)

        rgba = image.convert("RGBA")
        dhash = dhash_image(rgba)
        dhashes[type_id] = dhash
        locks[type_id] = {"approved_sha256": digest, "approved_dhash": dhash}

        expected_dhash = ship["sprite"].get("approved_dhash")
        if expected_dhash and dhash.lower() != expected_dhash.lower():
            add(findings, "error", "APPROVED_DHASH_CHANGED",
                f"{ship['name']} visually differs from its approved dHash.",
                rel_path, type_id=type_id)

        alpha = rgba.getchannel("A")
        extrema = alpha.getextrema()
        if ship["sprite"].get("alpha_required", True) and extrema == (255, 255):
            add(findings, "error", "NO_TRANSPARENCY",
                f"{ship['name']} has no transparent pixels.", rel_path, type_id=type_id)

        bbox = alpha.getbbox()
        if bbox is None:
            add(findings, "error", "EMPTY_SPRITE",
                f"{ship['name']} contains no visible pixels.", rel_path, type_id=type_id)
            continue

        width, height = rgba.size
        left, top, right, bottom = bbox
        if left == 0 or top == 0 or right == width or bottom == height:
            add(findings, "warning", "CONTENT_TOUCHES_EDGE",
                f"{ship['name']} visible pixels touch the canvas edge; clipping is likely.",
                rel_path, type_id=type_id)

        corner_alpha = [
            alpha.getpixel((0, 0)),
            alpha.getpixel((width - 1, 0)),
            alpha.getpixel((0, height - 1)),
            alpha.getpixel((width - 1, height - 1)),
        ]
        if max(corner_alpha) > 12:
            add(findings, "warning", "OPAQUE_CORNERS",
                f"{ship['name']} has nontransparent corner pixels.",
                rel_path, type_id=type_id)

        expected_canvas = ship["sprite"].get("expected_canvas")
        if expected_canvas and list(rgba.size) != expected_canvas:
            add(findings, "error", "CANVAS_SIZE_MISMATCH",
                f"{ship['name']} is {rgba.size[0]}×{rgba.size[1]}, "
                f"expected {expected_canvas[0]}×{expected_canvas[1]}.",
                rel_path, type_id=type_id)

    for digest, ids in exact_hashes.items():
        if len(ids) > 1:
            names = ", ".join(f"{manifest_by_id[i]['name']} ({i})" for i in ids)
            add(findings, "error", "EXACT_DUPLICATE_IMAGES",
                f"These hulls use byte-identical sprite files: {names}.")

    ids = sorted(dhashes)
    for index, left_id in enumerate(ids):
        for right_id in ids[index + 1:]:
            if exact_hashes and sha256_file(root / manifest_by_id[left_id]["sprite"]["path"]) == \
                    sha256_file(root / manifest_by_id[right_id]["sprite"]["path"]):
                continue
            distance = hamming_hex(dhashes[left_id], dhashes[right_id])
            if distance <= 2:
                add(findings, "warning", "NEAR_DUPLICATE_IMAGES",
                    f"{manifest_by_id[left_id]['name']} ({left_id}) and "
                    f"{manifest_by_id[right_id]['name']} ({right_id}) have near-identical "
                    f"perceptual hashes (distance {distance}). Verify visually.")

    for type_id, path in sorted(asset_pngs.items()):
        if type_id not in manifest_by_id:
            add(findings, "error", "ORPHAN_SPRITE",
                f"Numeric sprite {path.relative_to(root)} is not declared in the manifest.",
                path.relative_to(root), type_id=type_id)

    references, named_arms = collect_source_references(root)
    for type_id, locations in sorted(references.items()):
        if type_id not in manifest_by_id:
            first = locations[0]
            add(findings, "error", "UNREGISTERED_HULL_REFERENCE",
                f"Hull-like type ID {type_id} is referenced in code/config but absent "
                f"from the manifest. First context: {first[2]}",
                first[0], first[1], type_id)

    for type_id, expected_ship in sorted(manifest_by_id.items()):
        if type_id not in references:
            add(findings, "info", "MANIFEST_HULL_NOT_REFERENCED",
                f"{expected_ship['name']} is declared but no explicit source reference was found.",
                type_id=type_id)

    for type_id, code_name, code_class, line in named_arms:
        ship = manifest_by_id.get(type_id)
        if not ship:
            continue
        if code_name.casefold() != ship["name"].casefold():
            add(findings, "error", "CODE_NAME_MISMATCH",
                f"Code names type {type_id} {code_name!r}; manifest names it "
                f"{ship['name']!r}.", "src/entities/enemy/spawn.rs", line, type_id)
        expected_class = ship["class"]
        normalized_code_class = CLASS_MAP.get(code_class)
        if normalized_code_class and normalized_code_class != expected_class:
            # Specialized classes can legitimately collapse to their broad combat class.
            broad_compatible = (
                expected_class in {"assault_frigate", "interceptor"} and normalized_code_class == "frigate"
            ) or (
                expected_class in {"tactical_destroyer", "command_destroyer"}
                and normalized_code_class == "destroyer"
            ) or (
                expected_class == "heavy_assault_cruiser" and normalized_code_class == "cruiser"
            )
            if not broad_compatible:
                add(findings, "error", "CODE_CLASS_MISMATCH",
                    f"Code classifies {ship['name']} ({type_id}) as "
                    f"{normalized_code_class}; manifest says {expected_class}.",
                    "src/entities/enemy/spawn.rs", line, type_id)

    faction_path = root / "src/entities/enemy/faction.rs"
    inspect_mapping_function(
        root, faction_path, "get_enemy_color", manifest_by_id, findings
    )
    inspect_mapping_function(
        root, faction_path, "get_faction_engine_trail", manifest_by_id, findings
    )

    if Image is None:
        add(findings, "warning", "PILLOW_MISSING",
            "Pillow is not installed; image validation was skipped.")

    stats = {
        "manifest_hulls": len(manifest_by_id),
        "asset_pngs": len(asset_pngs),
        "source_reference_ids": len(references),
    }

    report = markdown_report(findings, stats)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(report + "\n", encoding="utf-8")

    json_report_path.parent.mkdir(parents=True, exist_ok=True)
    json_report_path.write_text(json.dumps({
        "stats": stats,
        "findings": [asdict(item) for item in findings],
        "locks": locks,
    }, indent=2) + "\n", encoding="utf-8")

    if contact_path:
        build_contact_sheet(root, ships, contact_path, findings)

    if args.print_locks:
        print(json.dumps(locks, indent=2))

    print(report)
    errors = sum(1 for item in findings if item.severity == "error")
    warnings = sum(1 for item in findings if item.severity == "warning")
    if errors:
        return 2
    if args.strict and warnings:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
