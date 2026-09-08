#!/usr/bin/env python3
"""Compare the hull registry with a dated CCP identity snapshot (offline).

Names and IDs are checked separately from visual identity; matching names do
not approve art. Refresh the snapshot from CCP ESI when changing the roster.
Exits 1 while mismatches remain, so a release gate cannot report a false pass.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def main():
    manifest = json.loads((ROOT / 'assets/ships/ship_manifest.json').read_text())
    reference = json.loads((ROOT / 'docs/ship-review/ccp-type-reference.json').read_text())
    types = {row['type_id']: row for row in reference['types']}
    mismatches = []
    for hull in manifest['ships']:
        official = types.get(hull['type_id'])
        if official is None or official['name'] != hull['name']:
            mismatches.append((hull['type_id'], hull['name'], official['name'] if official else 'NOT CHECKED'))
    for type_id, claimed, official in mismatches:
        print(f'{type_id}: registry={claimed}; CCP={official}')
    print(f'{len(manifest["ships"])} hulls checked; {len(mismatches)} identity mismatches; snapshot {reference["checked_on"]}')
    return int(bool(mismatches))

if __name__ == '__main__':
    raise SystemExit(main())
