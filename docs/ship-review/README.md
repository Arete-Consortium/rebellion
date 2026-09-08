# Ship review archive

This directory preserves the four historical ship-normalization iterations and
their contact sheets while keeping review material out of the game's runtime
`assets/` directory. Every original image and metadata sidecar can be recovered
byte for byte through [manifest.json](manifest.json).

The material arrived together in commit
`6d91714` on July 25, 2026. All iterations remain unapproved review candidates.
The live [ship manifest](../../assets/ships/ship_manifest.json) and its review
statuses were preserved unchanged. The highest-numbered iteration, v4, gets
storage priority when files are identical; its name does not establish visual
approval or authority over the runtime sprites.

## What was retained

The index maps each original repository path to a stored file, its byte count,
and its SHA-256 digest. PNGs and JSON sidecars were deduplicated only when their
complete file bytes matched. All unique variants and all four different contact
sheets remain. No symlinks are required.

| Iteration | Original files | Unique stored files | Stored bytes | Identical files represented by the index |
|---|---:|---:|---:|---:|
| `ships_normalized_v4` | 146 | 146 | 7,876,784 | 0 |
| `ships_normalized_v3` | 146 | 8 | 994,689 | 138 |
| `ships_normalized_v2` | 146 | 137 | 8,322,358 | 9 |
| `ships_normalized` | 146 | 146 | 7,103,805 | 0 |
| Contact sheets | 4 | 4 | 7,029,069 | 0 |
| **Total** | **588** | **441** | **31,326,705** | **147** |

The original material occupied **38,349,930 bytes**. Exact deduplication removed
**7,023,225 bytes** while preserving all 588 original file identities. Moving the
material removed **38,349,930 bytes** of review data from runtime `assets/`.
The index is additional metadata; these figures describe image and sidecar
content only.

The following separate, reproducible audit outputs were removed from tracking
and added to `.gitignore`: `build/ship-audit.md`, `build/ship-audit.json`, and
`build/ship-contact-sheet.png` (**1,162,054 bytes** combined). The accidental empty
root file named `=` was also removed. Total removed duplicate/generated content:
**8,185,279 bytes**, before the small new index, documentation, and recovery tool.

## Verify or recover an iteration

Run from the repository root; the recovery tool uses only Python's standard
library:

```bash
python3 scripts/materialize_ship_review.py --verify-only
python3 scripts/materialize_ship_review.py --output build/ship-review/restored
```

The second command reconstructs all original paths beneath the output directory,
for example `build/ship-review/restored/assets/ships_normalized_v3/587.png`.
It verifies every source hash before writing and refuses to overwrite different
existing content. The generated `build/ship-review/` directory is ignored by Git.

Cleanup validation reconstructed all **588 files** in a temporary directory and
compared every file byte for byte with its original blob in commit
`226f3bcdf5778cde60927e5d720c6fdc91de6f64`. All matched. A second reconstruction
accepted identical existing files; an intentionally conflicting destination was
rejected without overwriting it. The archive also passed a check that its 441
stored files exactly match the index, with no missing or unindexed content.

The live normalization script contains manual rotation overrides and mixed v2/v3
labels, and historical dependency versions were not recorded. Exact regeneration
of these historical iterations from the current algorithm has not been proven.
Use the archive recovery command when exact historical bytes matter.

## Browse the retained contact sheets

- [Original normalization](contact-sheets/ships_audit_contact_sheet.png)
- [v2](contact-sheets/ships_audit_v2_contact_sheet.png)
- [v3](contact-sheets/ships_audit_v3_contact_sheet.png)
- [v4](contact-sheets/ships_audit_v4_contact_sheet.png)

## Generate new review output

The input sprite paths and normalization algorithm remain unchanged. New
normalization output and orientation previews default to `build/ship-review/`:

```bash
python3 scripts/ship_normalizer.py
python3 scripts/analyze_ship_orientation.py --preview
```

Regenerate the current hull audit with the existing documented command:

```bash
python3 scripts/audit_ship_hulls.py \
  --root . \
  --manifest assets/ships/ship_manifest.json \
  --schema assets/ships/ship_manifest.schema.json \
  --report build/ship-audit.md \
  --json-report build/ship-audit.json \
  --contact-sheet build/ship-contact-sheet.png
```

Keep new review work here or in the ignored build output until its identity,
orientation, crop, and hardpoints have been approved. Current runtime loading
continues to use `assets/ships/{type_id}.png`; this cleanup does not promote any
normalized candidate into the game.

## Model source separation

The 11 source STL models and their Blender converter were subsequently moved to
[`tools/ship-models/`](../../tools/ship-models/). All STL bytes and existing runtime
GLBs were preserved; the converter's output path still targets `assets/models/`.
The tool README documents the generation command. This removes another
**2,461,819 bytes** of editable sources and tooling from runtime assets.

After both moves, the complete `assets/` tree decreased from **54,516,725 bytes**
to **13,704,976 bytes**, a reduction of **40,811,749 bytes**. This is asset-tree
size, separate from compressed package size. The 588-file image archive and its
verification provenance above remain unchanged.
