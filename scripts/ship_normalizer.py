#!/usr/bin/env python3
"""
Ship Asset Normalizer v2 for Rebellion.

Improvements over v1:
- Principal-axis rotation with nose-detection heuristics
- Manual override list for known problematic ships
- Smarter alpha cleaning: only cleans near-transparent edge pixels, preserves opaque dark hull detail
- Colored background detection: identifies dominant background color from border pixels and removes it
- Better handling of wide-wing ships (carriers, titans)

Usage:
    python3 scripts/ship_normalizer.py --input assets/ships --output assets/ships_normalized_v2
"""

import argparse
import json
import math
import os
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional, Tuple, List, Dict

import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter


# ─── Resolution Standards ───
CLASS_CANVAS = {
    "fighter": 256,
    "frigate": 256,
    "destroyer": 512,
    "cruiser": 512,
    "battlecruiser": 768,
    "battleship": 1024,
    "carrier": 1024,
    "dreadnought": 1024,
    "titan": 2048,
}

# EVE ship type_id → role class heuristic
DEFAULT_CLASS_BY_TYPE = {
    # Frigates
    582: "frigate", 583: "frigate", 584: "frigate", 585: "frigate",
    586: "frigate", 587: "frigate", 588: "frigate", 589: "frigate",
    590: "frigate", 591: "frigate", 592: "frigate", 593: "frigate",
    594: "frigate", 595: "frigate", 596: "frigate", 597: "frigate",
    598: "frigate", 599: "frigate", 600: "frigate", 601: "frigate",
    602: "frigate", 603: "frigate", 604: "frigate", 605: "frigate",
    608: "frigate", 609: "frigate", 610: "frigate", 611: "frigate",
    612: "frigate", 613: "frigate", 614: "frigate", 615: "frigate",
    617: "frigate", 618: "frigate", 619: "frigate", 620: "frigate",
    621: "frigate", 622: "frigate", 623: "frigate", 624: "frigate",
    625: "frigate", 626: "frigate", 628: "frigate", 629: "frigate",
    630: "frigate", 631: "frigate", 632: "frigate", 633: "frigate",
    634: "frigate", 635: "frigate", 638: "frigate", 639: "frigate",
    640: "frigate", 641: "frigate", 643: "frigate", 644: "frigate",
    645: "frigate",
    # Destroyers
    16236: "destroyer", 16238: "destroyer", 16240: "destroyer", 16242: "destroyer",
    32872: "destroyer", 32874: "destroyer", 32876: "destroyer", 32878: "destroyer",
    33818: "destroyer", 34317: "destroyer",
    # Cruisers
    620: "cruiser", 621: "cruiser", 622: "cruiser", 623: "cruiser",
    624: "cruiser", 625: "cruiser", 626: "cruiser", 627: "cruiser",
    630: "cruiser", 631: "cruiser", 632: "cruiser", 633: "cruiser",
    634: "cruiser", 635: "cruiser", 638: "cruiser", 639: "cruiser",
    640: "cruiser", 641: "cruiser", 642: "cruiser", 643: "cruiser",
    644: "cruiser", 645: "cruiser",
    # Battleships
    24688: "battleship", 24690: "battleship", 24692: "battleship",
    24694: "battleship", 24696: "battleship", 24698: "battleship",
    24700: "battleship", 24702: "battleship",
    # Carriers / Capitals
    23911: "carrier", 23915: "carrier", 24483: "carrier",
    3764: "titan",  # Leviathan
    671: "titan",   # Erebus
    11567: "titan",
    # Special / Boss
    1944: "frigate", 2006: "frigate", 11184: "cruiser",
    11993: "cruiser", 12019: "cruiser",
    11547: "battleship", 11566: "battleship", 11568: "battleship",
    17713: "battleship", 20185: "battleship", 23757: "battleship",
    47269: "frigate", 47270: "frigate", 47271: "frigate",
    49710: "frigate", 49711: "frigate",
    52250: "frigate", 54731: "frigate", 54732: "frigate", 54733: "frigate",
}

# Ships known to be horizontal in source (side-view renders)
HORIZONTAL_SHIPS = {23911, 23915, 24483}  # Carriers

# Ships that need manual rotation correction (type_id -> final rotation degrees)
# Positive = clockwise, Negative = counter-clockwise
MANUAL_ROTATION_OVERRIDES = {
    16238: -45,   # Destroyer at odd 45° angle
    608: -45,     # Destroyer at odd 138° angle
    3764: 0,      # Leviathan is already vertical
    583: -90,     # Horizontal frigate pointing left
}


@dataclass
class ShipMetadata:
    type_id: int
    class_: str
    source_size: Tuple[int, int]
    output_size: int
    rotation_degrees: float
    visual_center: List[float]
    engine_hardpoints: List[List[float]]
    weapon_hardpoints: List[List[float]]
    collision_scale: float
    shadow_scale: float
    display_scale: float


def load_source(path: Path) -> Image.Image:
    """Load a source PNG, ensuring RGBA mode."""
    img = Image.open(path)
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    return img


def detect_colored_background(img: Image.Image) -> Optional[Tuple[int, int, int]]:
    """
    Detect a colored background using multiple strategies:
    1. Sample low-alpha pixels (for transparent-background images)
    2. Sample corner pixels (for opaque studio-render backgrounds)
    3. Sample border pixels (gradient backgrounds)
    Returns (R, G, B) of background or None if no clear background.
    """
    arr = np.array(img)
    h, w = arr.shape[:2]

    all_samples = []

    # Strategy 1: Low-alpha pixels (transparent/gradient backgrounds)
    alpha = arr[:, :, 3]
    low_alpha_mask = alpha < 80
    if np.any(low_alpha_mask):
        all_samples.append(arr[low_alpha_mask][:, :3])

    # Strategy 2: Corner pixels (opaque studio renders)
    # The four corners are almost certainly background
    corner_size = max(1, int(min(w, h) * 0.08))
    corners = [
        arr[:corner_size, :corner_size, :3],           # Top-left
        arr[:corner_size, -corner_size:, :3],          # Top-right
        arr[-corner_size:, :corner_size, :3],          # Bottom-left
        arr[-corner_size:, -corner_size:, :3],         # Bottom-right
    ]
    for corner in corners:
        all_samples.append(corner.reshape(-1, 3))

    # Strategy 3: Border ring
    border_pixels = []
    border_thickness = max(1, int(min(w, h) * 0.03))
    for x in range(w):
        for y in range(border_thickness):
            border_pixels.append(arr[y, x, :3])
        for y in range(h - border_thickness, h):
            border_pixels.append(arr[y, x, :3])
    for y in range(border_thickness, h - border_thickness):
        for x in range(border_thickness):
            border_pixels.append(arr[y, x, :3])
        for x in range(w - border_thickness, w):
            border_pixels.append(arr[y, x, :3])
    if border_pixels:
        all_samples.append(np.array(border_pixels))

    if not all_samples:
        return None

    combined = np.vstack(all_samples)

    # Find most common color (with tolerance)
    rounded = (combined // 24) * 24
    colors, counts = np.unique(rounded.reshape(-1, 3), axis=0, return_counts=True)

    if len(counts) == 0:
        return None

    dominant_idx = np.argmax(counts)
    dominant = colors[dominant_idx]
    dominant_count = counts[dominant_idx]
    total = len(combined)

    # Only declare a background if it dominates (>30% of samples)
    if dominant_count / total < 0.30:
        return None

    # Return the actual average color of that cluster
    mask = np.all(np.abs(combined - dominant) <= 36, axis=1)
    if not np.any(mask):
        return None
    actual = combined[mask].mean(axis=0).astype(int)
    return tuple(actual)


def remove_colored_background(img: Image.Image, bg_color: Tuple[int, int, int]) -> Image.Image:
    """
    Aggressively remove pixels matching the detected background color.
    Handles gradient backgrounds by using wider tolerance.
    """
    arr = np.array(img)
    r, g, b, a = arr[:, :, 0], arr[:, :, 1], arr[:, :, 2], arr[:, :, 3]

    bg_r, bg_g, bg_b = bg_color

    # Primary: exact color match with wide tolerance (for gradients)
    color_match = (
        (np.abs(r.astype(int) - bg_r) < 45) &
        (np.abs(g.astype(int) - bg_g) < 45) &
        (np.abs(b.astype(int) - bg_b) < 45)
    )
    arr[color_match] = [0, 0, 0, 0]

    # Secondary: near-background pixels with low/medium alpha
    # These are usually gradient edges or anti-aliased halo
    weak_alpha = a < 150
    near_bg = (
        (np.abs(r.astype(int) - bg_r) < 70) &
        (np.abs(g.astype(int) - bg_g) < 70) &
        (np.abs(b.astype(int) - bg_b) < 70)
    )
    arr[weak_alpha & near_bg] = [0, 0, 0, 0]

    # Tertiary: for very bright backgrounds (like white/blue studio renders),
    # also catch any pixel that is close to background AND the ship is dark
    dark_ship = (r < 80) & (g < 80) & (b < 80)
    bright_bg = (bg_r > 80) or (bg_g > 80) or (bg_b > 80)
    if bright_bg:
        bg_edge = (
            (np.abs(r.astype(int) - bg_r) < 60) &
            (np.abs(g.astype(int) - bg_g) < 60) &
            (np.abs(b.astype(int) - bg_b) < 60) &
            ~dark_ship
        )
        arr[bg_edge] = [0, 0, 0, 0]

    return Image.fromarray(arr, "RGBA")


def clean_alpha_v2(img: Image.Image) -> Image.Image:
    """
    Smarter alpha cleaning:
    - Only removes near-transparent dark pixels (< 30 alpha)
    - Preserves opaque dark hull detail
    - Removes gray/black halos at semi-transparent edges
    """
    arr = np.array(img)
    r, g, b, a = arr[:, :, 0], arr[:, :, 1], arr[:, :, 2], arr[:, :, 3]

    # Near-transparent pixels with any dark color -> transparent
    near_transparent = a < 30
    dark = (r < 60) & (g < 60) & (b < 60)
    arr[near_transparent & dark] = [0, 0, 0, 0]

    # Semi-transparent edge pixels (30-150 alpha) that are very dark
    # These are usually halo artifacts
    semi_transparent = (a >= 30) & (a < 150)
    very_dark = (r < 25) & (g < 25) & (b < 25)
    arr[semi_transparent & very_dark] = [0, 0, 0, 0]

    # For semi-transparent pixels that are moderately dark, reduce alpha
    # instead of zeroing (preserves some anti-aliasing)
    moderate_dark = (r < 50) & (g < 50) & (b < 50)
    halo_mask = semi_transparent & moderate_dark & ~very_dark
    arr[halo_mask, 3] = (arr[halo_mask, 3] * 0.5).astype(np.uint8)

    return Image.fromarray(arr, "RGBA")


def detect_opaque_bbox(img: Image.Image) -> Tuple[int, int, int, int]:
    """Return (left, top, right, bottom) bounding box of opaque pixels."""
    arr = np.array(img)
    alpha = arr[:, :, 3]
    mask = alpha > 15
    rows = np.any(mask, axis=1)
    cols = np.any(mask, axis=0)
    if not np.any(rows) or not np.any(cols):
        return (0, 0, img.width, img.height)
    top = int(np.argmax(rows))
    bottom = int(len(rows) - np.argmax(rows[::-1]))
    left = int(np.argmax(cols))
    right = int(len(cols) - np.argmax(cols[::-1]))
    return (left, top, right, bottom)


def compute_principal_axis(arr: np.ndarray) -> Tuple[float, float, float]:
    """
    Compute principal axis of silhouette.
    Returns (angle_degrees, cx, cy) where angle is 0 when axis is vertical (nose up).
    """
    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 15)
    if len(xs) < 10:
        return 0.0, arr.shape[1] / 2, arr.shape[0] / 2

    cx = float(np.mean(xs))
    cy = float(np.mean(ys))

    x_centered = xs - cx
    y_centered = ys - cy

    cov_xx = float(np.mean(x_centered ** 2))
    cov_yy = float(np.mean(y_centered ** 2))
    cov_xy = float(np.mean(x_centered * y_centered))

    # Eigenvalues
    trace = cov_xx + cov_yy
    det = cov_xx * cov_yy - cov_xy * cov_xy
    sqrt_term = math.sqrt(max(trace**2 - 4*det, 0))
    eig1 = (trace + sqrt_term) / 2
    eig2 = (trace - sqrt_term) / 2

    # Eigenvector for largest eigenvalue (major axis)
    if abs(cov_xy) < 1e-6:
        if cov_xx > cov_yy:
            angle = 90.0  # Horizontal major axis
        else:
            angle = 0.0   # Vertical major axis
    else:
        vx = eig1 - cov_yy
        vy = cov_xy
        angle = math.degrees(math.atan2(vy, vx))

    return angle, cx, cy


def determine_nose_direction(arr: np.ndarray, angle: float, cx: float, cy: float) -> int:
    """
    Determine which end of the principal axis is the nose.
    Returns +1 if the top end is the nose, -1 if bottom end is nose.
    Heuristic: the nose is usually narrower (fewer pixels perpendicular to axis).
    """
    alpha = arr[:, :, 3]
    h, w = alpha.shape

    # Convert angle to unit vector along the axis
    rad = math.radians(angle)
    ux = math.cos(rad)
    uy = math.sin(rad)

    # Perpendicular unit vector
    px = -uy
    py = ux

    # Project all silhouette pixels onto the axis
    ys, xs = np.where(alpha > 15)
    if len(xs) < 10:
        return 1

    # Centered coordinates
    dx = xs - cx
    dy = ys - cy

    # Projection along axis
    proj = dx * ux + dy * uy

    # Perpendicular spread (width) at each end
    top_mask = proj > 0
    bottom_mask = proj < 0

    if not np.any(top_mask) or not np.any(bottom_mask):
        return 1

    # Compute perpendicular spread for each half
    perp_top = dx[top_mask] * px + dy[top_mask] * py
    perp_bottom = dx[bottom_mask] * px + dy[bottom_mask] * py

    spread_top = np.std(perp_top) if len(perp_top) > 1 else 0
    spread_bottom = np.std(perp_bottom) if len(perp_bottom) > 1 else 0

    # The nose is the narrower end
    if spread_top < spread_bottom:
        return 1  # Top is nose
    else:
        return -1  # Bottom is nose


def determine_rotation_v2(img: Image.Image, type_id: int) -> float:
    """
    Determine rotation to make ship nose point up.
    Uses principal axis + nose detection.
    """
    # Manual override takes precedence
    if type_id in MANUAL_ROTATION_OVERRIDES:
        return MANUAL_ROTATION_OVERRIDES[type_id]

    arr = np.array(img)
    angle, cx, cy = compute_principal_axis(arr)

    # The angle from compute_principal_axis is the major axis direction
    # If the ship is horizontal in source, angle ~ 90°
    # We want the ship vertical (nose up), so we rotate by -angle

    # But first, determine nose direction
    nose_dir = determine_nose_direction(arr, angle, cx, cy)

    # If nose points down, add 180° flip
    if nose_dir == -1:
        angle += 180

    # For horizontal ships, the major axis is at 90°.
    # To make it vertical, we rotate by -90°.
    # For ships already roughly vertical, angle is ~0°, so rotation is ~0°.
    rotation = -angle

    # Normalize to [-180, 180]
    while rotation > 180:
        rotation -= 360
    while rotation < -180:
        rotation += 360

    # Special case: if the ship is in HORIZONTAL_SHIPS set, force -90
    if type_id in HORIZONTAL_SHIPS:
        rotation = -90

    return rotation


def normalize_ship_v2(
    img: Image.Image,
    type_id: int,
    canvas_size: int,
) -> Tuple[Image.Image, ShipMetadata]:
    """
    Normalize a ship image to the target canvas.
    Returns (normalized_image, metadata).
    """
    # Step 1: Detect and remove colored background
    bg_color = detect_colored_background(img)
    if bg_color:
        img = remove_colored_background(img, bg_color)

    # Step 2: Clean alpha (smart edge cleaning)
    img = clean_alpha_v2(img)

    # Step 3: Detect opaque bounding box
    bbox = detect_opaque_bbox(img)
    left, top, right, bottom = bbox
    bbox_w = right - left
    bbox_h = bottom - top

    # Step 4: Determine rotation
    rotation = determine_rotation_v2(img, type_id)

    # Step 5: Rotate the full image
    if abs(rotation) > 0.1:
        # Expand canvas to avoid clipping corners
        # Calculate required expansion
        diag = math.sqrt(img.width**2 + img.height**2)
        pad = int((diag - max(img.width, img.height)) / 2) + 10
        padded = Image.new("RGBA", (img.width + pad*2, img.height + pad*2), (0, 0, 0, 0))
        padded.paste(img, (pad, pad), img)

        rotated = padded.rotate(rotation, resample=Image.BICUBIC, expand=True)

        # Recompute bbox after rotation
        bbox = detect_opaque_bbox(rotated)
        rl, rt, rr, rb = bbox
        # Crop to bbox
        img = rotated.crop((rl, rt, rr, rb))
        left, top, right, bottom = 0, 0, img.width, img.height
        bbox_w, bbox_h = img.width, img.height
    else:
        # Just crop to bbox
        img = img.crop((left, top, right, bottom))
        left, top, right, bottom = 0, 0, img.width, img.height
        bbox_w, bbox_h = img.width, img.height

    # Step 6: Scale to fit canvas with padding
    max_dimension = max(bbox_w, bbox_h)
    target_size = int(canvas_size * 0.84)  # 8% padding each side
    scale = target_size / max_dimension if max_dimension > 0 else 1.0
    new_w = max(1, int(bbox_w * scale))
    new_h = max(1, int(bbox_h * scale))

    scaled = img.resize((new_w, new_h), Image.LANCZOS)

    # Step 7: Center on canvas
    canvas = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    paste_x = (canvas_size - new_w) // 2
    paste_y = (canvas_size - new_h) // 2
    canvas.paste(scaled, (paste_x, paste_y), scaled)

    # Step 8: Compute metadata
    arr = np.array(canvas)
    alpha = arr[:, :, 3]
    ys, xs = np.where(alpha > 15)
    if len(xs) > 0:
        vis_cx = (float(np.mean(xs)) - canvas_size / 2) / (canvas_size / 2)
        vis_cy = (float(np.mean(ys)) - canvas_size / 2) / (canvas_size / 2)
    else:
        vis_cx, vis_cy = 0.0, 0.0

    class_ = DEFAULT_CLASS_BY_TYPE.get(type_id, "frigate")

    # Hardpoints based on ship class size
    if class_ in ("carrier", "titan", "dreadnought"):
        engine_y = 0.40
        engine_x = 0.25
        weapon_y = -0.15
        weapon_x = 0.30
    elif class_ in ("battleship", "battlecruiser"):
        engine_y = 0.38
        engine_x = 0.22
        weapon_y = -0.12
        weapon_x = 0.28
    elif class_ == "cruiser":
        engine_y = 0.36
        engine_x = 0.20
        weapon_y = -0.10
        weapon_x = 0.25
    elif class_ == "destroyer":
        engine_y = 0.35
        engine_x = 0.19
        weapon_y = -0.10
        weapon_x = 0.23
    else:  # frigate/fighter
        engine_y = 0.35
        engine_x = 0.18
        weapon_y = -0.10
        weapon_x = 0.22

    engine_hardpoints = [[-engine_x, engine_y], [engine_x, engine_y]]
    weapon_hardpoints = [[-weapon_x, weapon_y], [weapon_x, weapon_y]]

    # Scale factors by class
    collision_scale = 0.52
    shadow_scale = 0.85
    display_scale = 1.0

    metadata = ShipMetadata(
        type_id=type_id,
        class_=class_,
        source_size=(img.width, img.height),
        output_size=canvas_size,
        rotation_degrees=round(rotation, 2),
        visual_center=[round(vis_cx, 3), round(vis_cy, 3)],
        engine_hardpoints=engine_hardpoints,
        weapon_hardpoints=weapon_hardpoints,
        collision_scale=collision_scale,
        shadow_scale=shadow_scale,
        display_scale=display_scale,
    )

    return canvas, metadata


def generate_contact_sheet(
    images: List[Tuple[int, Image.Image, ShipMetadata]],
    output_path: Path,
    columns: int = 8,
    thumb_size: int = 256,
) -> None:
    """Generate a visual contact sheet of all normalized ships."""
    count = len(images)
    rows = math.ceil(count / columns)

    margin = 10
    label_height = 35
    cell_w = thumb_size + margin * 2
    cell_h = thumb_size + label_height + margin * 2

    sheet_w = cell_w * columns + margin
    sheet_h = cell_h * rows + margin + 30  # Extra for header

    sheet = Image.new("RGB", (sheet_w, sheet_h), (20, 20, 25))
    draw = ImageDraw.Draw(sheet)

    try:
        font_label = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 12)
        font_header = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 18)
    except:
        font_label = ImageFont.load_default()
        font_header = ImageFont.load_default()

    # Header
    header_text = f"SHIP ASSET AUDIT v3 — {count} hulls normalized"
    draw.text((margin, 8), header_text, font=font_header, fill=(170, 221, 170))

    for idx, (type_id, img, meta) in enumerate(images):
        row = idx // columns
        col = idx % columns
        x = margin + col * cell_w
        y = margin + 30 + row * cell_h

        # Cell background
        draw.rectangle(
            [x, y, x + cell_w - margin, y + cell_h - margin],
            outline=(40, 60, 40),
            fill=(10, 12, 15),
        )

        # Thumbnail (scale to fit thumb_size)
        thumb = img.resize((thumb_size, thumb_size), Image.LANCZOS)
        thumb_bg = Image.new("RGB", (thumb_size, thumb_size), (10, 12, 15))
        thumb_bg.paste(thumb, (0, 0), thumb)
        sheet.paste(thumb_bg, (x + margin, y + margin))

        # Label
        label = f"{type_id} | {meta.class_} | {meta.output_size}px"
        if meta.rotation_degrees != 0:
            label += f" | rot:{meta.rotation_degrees:.0f}°"
        draw.text((x + margin, y + thumb_size + margin + 5), label, font=font_label, fill=(136, 170, 136))

        # Show rotation indicator if rotated
        if abs(meta.rotation_degrees) > 1:
            indicator = "↻" if meta.rotation_degrees > 0 else "↺"
            draw.text((x + cell_w - margin - 20, y + 5), indicator, font=font_label, fill=(255, 200, 100))

    sheet.save(output_path, "PNG", optimize=True)
    print(f"  Contact sheet saved: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Normalize ship assets for Rebellion v2")
    parser.add_argument("--input", default="assets/ships", help="Source ship image directory")
    parser.add_argument("--output", default="assets/ships_normalized_v2", help="Output directory")
    parser.add_argument("--contact-sheet", default="assets/ships_audit_v2_contact_sheet.png", help="Contact sheet output path")
    parser.add_argument("--config", default="", help="Optional JSON config for type_id→class overrides")
    args = parser.parse_args()

    input_dir = Path(args.input)
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)
    metadata_dir = output_dir / "metadata"
    metadata_dir.mkdir(parents=True, exist_ok=True)

    # Load optional class overrides
    if args.config and Path(args.config).exists():
        with open(args.config) as f:
            overrides = json.load(f)
            DEFAULT_CLASS_BY_TYPE.update(overrides)

    images: List[Tuple[int, Image.Image, ShipMetadata]] = []
    skipped = 0

    for filename in sorted(input_dir.glob("*.png")):
        stem = filename.stem
        try:
            type_id = int(stem)
        except ValueError:
            print(f"  Skip non-numeric: {filename.name}")
            skipped += 1
            continue

        class_ = DEFAULT_CLASS_BY_TYPE.get(type_id, "frigate")
        canvas_size = CLASS_CANVAS.get(class_, 512)

        print(f"Processing {type_id} ({class_}, {canvas_size}px canvas)...")
        img = load_source(filename)
        normalized, meta = normalize_ship_v2(img, type_id, canvas_size)

        out_path = output_dir / f"{type_id}.png"
        normalized.save(out_path, "PNG", optimize=True)

        meta_path = metadata_dir / f"{type_id}.json"
        with open(meta_path, "w") as f:
            json.dump(asdict(meta), f, indent=2)

        images.append((type_id, normalized, meta))

    print(f"\nProcessed {len(images)} ships, skipped {skipped} non-numeric files.")

    if images:
        generate_contact_sheet(images, Path(args.contact_sheet))

    print(f"\nOutput directories:")
    print(f"  Normalized ships: {output_dir}/")
    print(f"  Metadata JSON:    {metadata_dir}/")
    print(f"  Contact sheet:    {args.contact_sheet}")


if __name__ == "__main__":
    main()
