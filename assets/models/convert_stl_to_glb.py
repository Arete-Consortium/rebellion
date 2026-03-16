#!/usr/bin/env python3
"""
Blender script to convert EVE Online STL ship models to GLB format.
Run with: blender --background --python convert_stl_to_glb.py
"""
import bpy
import os
import sys
from pathlib import Path

# Ship configurations: (stl_filename, output_name, faction_color_rgb)
SHIPS = [
    # Minmatar - Rusty brown/red
    ("rifter.stl", "rifter.glb", (0.65, 0.35, 0.25)),
    ("slasher.stl", "slasher.glb", (0.65, 0.35, 0.25)),
    ("probe.stl", "probe.glb", (0.65, 0.35, 0.25)),
    ("breacher.stl", "breacher.glb", (0.65, 0.35, 0.25)),

    # Amarr - Gold/bronze
    ("punisher.stl", "punisher.glb", (0.85, 0.7, 0.35)),
    ("executioner.stl", "executioner.glb", (0.85, 0.7, 0.35)),
    ("tormentor.stl", "tormentor.glb", (0.85, 0.7, 0.35)),

    # Caldari - Steel blue/gray
    ("merlin.stl", "merlin.glb", (0.5, 0.55, 0.65)),
    ("kestrel.stl", "kestrel.glb", (0.5, 0.55, 0.65)),

    # Gallente - Dark green/teal
    ("tristan.stl", "tristan.glb", (0.3, 0.55, 0.45)),
    ("incursus.stl", "incursus.glb", (0.3, 0.55, 0.45)),
]

def create_faction_material(name: str, color: tuple) -> bpy.types.Material:
    """Create a PBR material with faction colors."""
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True

    nodes = mat.node_tree.nodes
    links = mat.node_tree.links

    # Clear default nodes
    nodes.clear()

    # Create Principled BSDF shader
    bsdf = nodes.new('ShaderNodeBsdfPrincipled')
    bsdf.location = (0, 0)

    # Set base color (faction color)
    bsdf.inputs['Base Color'].default_value = (*color, 1.0)

    # Metallic look for spaceships
    bsdf.inputs['Metallic'].default_value = 0.7

    # Some roughness for worn metal look
    bsdf.inputs['Roughness'].default_value = 0.4

    # Material output
    output = nodes.new('ShaderNodeOutputMaterial')
    output.location = (300, 0)

    # Connect BSDF to output
    links.new(bsdf.outputs['BSDF'], output.inputs['Surface'])

    return mat


def convert_stl_to_glb(stl_path: Path, glb_path: Path, color: tuple):
    """Convert a single STL file to GLB with faction material."""
    print(f"Converting: {stl_path.name} -> {glb_path.name}")

    # Clear the scene
    bpy.ops.wm.read_factory_settings(use_empty=True)

    # Import STL
    bpy.ops.wm.stl_import(filepath=str(stl_path))

    # Get the imported object
    if not bpy.context.selected_objects:
        print(f"  ERROR: No object imported from {stl_path.name}")
        return False

    obj = bpy.context.selected_objects[0]

    # Normalize scale (STL units vary)
    # Get bounding box and scale to fit within 1 unit
    dimensions = obj.dimensions
    max_dim = max(dimensions)
    if max_dim > 0:
        scale_factor = 1.0 / max_dim
        obj.scale = (scale_factor, scale_factor, scale_factor)

    # Apply scale
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    # Center the object
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    obj.location = (0, 0, 0)

    # Create and assign faction material
    mat_name = f"Material_{stl_path.stem}"
    material = create_faction_material(mat_name, color)

    if obj.data.materials:
        obj.data.materials[0] = material
    else:
        obj.data.materials.append(material)

    # Smooth shading for better appearance
    bpy.ops.object.shade_smooth()

    # Export to GLB
    bpy.ops.export_scene.gltf(
        filepath=str(glb_path),
        export_format='GLB',
        use_selection=False,
        export_apply=True,
        export_materials='EXPORT',
    )

    print(f"  SUCCESS: {glb_path.name} created")
    return True


def main():
    script_dir = Path(__file__).parent
    stl_dir = script_dir / "stl_source"
    output_dir = script_dir

    print(f"STL source: {stl_dir}")
    print(f"Output dir: {output_dir}")
    print(f"Converting {len(SHIPS)} ships...")
    print()

    success_count = 0
    for stl_name, glb_name, color in SHIPS:
        stl_path = stl_dir / stl_name
        glb_path = output_dir / glb_name

        if not stl_path.exists():
            print(f"SKIP: {stl_name} not found")
            continue

        if convert_stl_to_glb(stl_path, glb_path, color):
            success_count += 1

    print()
    print(f"Conversion complete: {success_count}/{len(SHIPS)} ships converted")


if __name__ == "__main__":
    main()
