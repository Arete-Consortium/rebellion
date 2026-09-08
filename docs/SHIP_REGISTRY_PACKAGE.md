# Rebellion Ship Registry Package

This package contains all three requested deliverables:

1. `assets/ships/ship_manifest.schema.json`
   - Strict JSON Schema for canonical ship visual identity.
2. `assets/ships/ship_manifest.starter.json`
   - Starter manifest populated from current Rebellion hull references.
3. `scripts/audit_ship_hulls.py`
   - Project-wide source, asset, image and identity audit.
4. `docs/SHIP_REGISTRY_REFACTOR_PLAN.md`
   - Ordered Rust migration plan.
5. `src/assets/ship_registry.rs.template`
   - Compile-ready starting point for the Bevy registry.

## Install

Copy the files into the repository, then rename:

```bash
cp assets/ships/ship_manifest.starter.json assets/ships/ship_manifest.json
python -m pip install pillow jsonschema
```

## First audit

```bash
python scripts/audit_ship_hulls.py \
  --root . \
  --manifest assets/ships/ship_manifest.json \
  --schema assets/ships/ship_manifest.schema.json \
  --report build/ship-audit.md \
  --json-report build/ship-audit.json \
  --contact-sheet build/ship-contact-sheet.png
```

The starter manifest deliberately marks every hull `pending_review`.
Do not mark a hull `approved` until its contact-sheet image, orientation,
centering, and identity have been checked.
