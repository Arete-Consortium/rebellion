# Ship model sources

This directory holds the 11 editable STL ship models and their Blender converter.
They were moved from `assets/models/` so source meshes and build tools are kept
outside the runtime asset tree. The existing 11 game-ready GLB files remain in
[`assets/models/`](../../assets/models/).

From the repository root, generate the GLB files with Blender:

```bash
blender --background --python tools/ship-models/convert_stl_to_glb.py
```

The converter reads `stl_source/` next to the script and writes to
`assets/models/` at the repository root. It overwrites the listed GLB outputs.
Its ship list, geometry conversion, faction materials, and export settings are
unchanged; only its location, documented command, and output path were adjusted.

The move preserved all 11 STL files byte for byte against their original Git
blobs. Existing GLB outputs were also verified unchanged. The moved source and
converter occupied **2,461,819 bytes** before the small path/documentation edit.
Blender conversion was not rerun as part of this relocation.
