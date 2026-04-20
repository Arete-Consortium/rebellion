#!/usr/bin/env python3
"""
Ship Orientation Analyzer
--------------------------
For each ship sprite in assets/ships/, compute:
 - principal axis (long axis of the ship silhouette)
 - which end is the nose (tapered, narrower)
 - the rotation (in radians) needed to point the nose to +Y (screen up)

Outputs a Rust-ready match arm block that can be pasted into
entities/enemy/faction.rs::get_ship_rotation_correction().

Usage:
    python3 scripts/analyze_ship_orientation.py [TYPE_ID ...]

Arguments are optional; no args = analyze every sprite in assets/ships.
"""
import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw
    import numpy as np
except ImportError:
    print("Requires pillow + numpy: pip install pillow numpy")
    sys.exit(1)

SHIPS_DIR = Path(__file__).resolve().parent.parent / "assets" / "ships"
PREVIEW_DIR = Path(__file__).resolve().parent.parent / "assets" / "_orientation_preview"


def load_mask(path: Path) -> np.ndarray:
    """Load sprite, return HxW bool mask of opaque pixels."""
    img = Image.open(path).convert("RGBA")
    alpha = np.array(img)[:, :, 3]
    # Treat anything with alpha > 40 as "ship pixel"
    return alpha > 40


def analyze(mask: np.ndarray):
    """
    Returns (rotation_radians, confidence) where rotation is the angle to add
    to the sprite so the nose points up (+Y in screen space).

    Method:
      1. PCA on opaque pixels → principal long axis.
      2. For each direction along the axis, count pixels in the extreme 20%.
         The nose is the SHARPER (fewer pixels) tip — ships taper to a point
         at the bow but have engines/hull mass at the tail.
      3. Fall back to pixel-count along axis if tie.
    """
    ys, xs = np.where(mask)
    if len(xs) < 50:
        return 0.0, 0.0

    cx, cy = xs.mean(), ys.mean()
    X = xs - cx
    Y = -(ys - cy)  # image y-axis is flipped
    pts = np.stack([X, Y], axis=1)

    cov = np.cov(pts.T)
    _, eigvecs = np.linalg.eigh(cov)
    principal = eigvecs[:, -1]

    proj = pts @ principal
    # CCP consistent render convention: ships are viewed 3/4 starboard with
    # the bow tilted toward the camera's upper-left. So the NOSE end of the
    # principal axis is the one that sits higher (larger +Y in math coords)
    # AND further left (smaller x). Pick the axis tip that projects MORE
    # strongly onto (-1, +1) — the "upper-left" direction.
    tip_sorted = np.argsort(proj)
    pos_tip_pts = pts[tip_sorted[-max(20, len(proj) // 10):]].mean(axis=0)
    neg_tip_pts = pts[tip_sorted[:max(20, len(proj) // 10)]].mean(axis=0)
    # Score each tip by "upper-left-ness": -x + y
    pos_score = -pos_tip_pts[0] + pos_tip_pts[1]
    neg_score = -neg_tip_pts[0] + neg_tip_pts[1]
    if pos_score > neg_score:
        nose_sign = +1.0
    else:
        nose_sign = -1.0
    asymmetry = abs(pos_score - neg_score) / (abs(pos_score) + abs(neg_score) + 1e-6)

    # The nose direction in ship-local space is principal * nose_sign.
    nose_dir = principal * nose_sign
    # Target is (0, +1) = screen up. Rotation needed = angle from nose_dir to +Y.
    current_angle = math.atan2(nose_dir[1], nose_dir[0])
    target_angle = math.pi / 2  # +Y
    rotation = target_angle - current_angle

    # Normalize to [-pi, pi]
    while rotation > math.pi:
        rotation -= 2 * math.pi
    while rotation < -math.pi:
        rotation += 2 * math.pi

    return rotation, asymmetry


def format_radians(r: float) -> str:
    """Format angle as a readable Rust constant or literal."""
    # Snap to common values
    candidates = [
        (0.0, "0.0"),
        (math.pi / 2, "FRAC_PI_2"),
        (-math.pi / 2, "-FRAC_PI_2"),
        (math.pi, "PI"),
        (-math.pi, "-PI"),
        (math.pi / 4, "FRAC_PI_4"),
        (-math.pi / 4, "-FRAC_PI_4"),
        (3 * math.pi / 4, "3.0 * FRAC_PI_4"),
        (-3 * math.pi / 4, "-3.0 * FRAC_PI_4"),
    ]
    best = min(candidates, key=lambda c: abs(c[0] - r))
    if abs(best[0] - r) < 0.25:  # within ~14° — good enough
        return best[1]
    return f"{r:.3f}"


def render_preview(src_path: Path, mask: np.ndarray, rotation: float, out_path: Path):
    """Draw the detected principal axis + nose arrow on top of the sprite so a
    human can verify whether the analyzer got it right."""
    img = Image.open(src_path).convert("RGBA")
    h, w = mask.shape
    draw = ImageDraw.Draw(img)
    ys, xs = np.where(mask)
    cx, cy = float(xs.mean()), float(ys.mean())

    # The rotation value is "angle to add so nose points up".
    # So current nose direction in sprite coords = rotate(+Y, -rotation).
    nose_angle = math.pi / 2 - rotation  # in math coords (y-up)
    dx = math.cos(nose_angle)
    dy = -math.sin(nose_angle)  # flip back to image coords (y-down)
    length = max(w, h) * 0.4
    tip_x = cx + dx * length
    tip_y = cy + dy * length
    # Arrow shaft
    draw.line([(cx, cy), (tip_x, tip_y)], fill=(0, 255, 0, 255), width=3)
    # Arrowhead
    draw.ellipse(
        [(tip_x - 6, tip_y - 6), (tip_x + 6, tip_y + 6)],
        fill=(0, 255, 0, 255),
    )
    # Centroid marker
    draw.ellipse(
        [(cx - 4, cy - 4), (cx + 4, cy + 4)], fill=(255, 255, 0, 255)
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    img.save(out_path)


def main():
    arg_ids = set(int(a) for a in sys.argv[1:] if a.isdigit()) or None
    write_previews = "--preview" in sys.argv

    results = []
    for path in sorted(SHIPS_DIR.glob("*.png")):
        try:
            tid = int(path.stem)
        except ValueError:
            continue
        if arg_ids is not None and tid not in arg_ids:
            continue
        try:
            mask = load_mask(path)
            rot, conf = analyze(mask)
            results.append((tid, rot, conf, mask.shape))
            if write_previews:
                out = PREVIEW_DIR / f"{tid}_preview.png"
                render_preview(path, mask, rot, out)
        except Exception as e:
            print(f"  {tid}: ERROR {e}", file=sys.stderr)

    print(f"# Ship rotation analysis ({len(results)} sprites)")
    print("# tid, suggested_rotation, confidence, (h, w)")
    print()
    for tid, rot, conf, shape in sorted(results):
        deg = math.degrees(rot)
        marker = " ⚠" if conf < 0.08 else ""
        print(f"  {tid:>6} => {format_radians(rot):<14}, // {deg:+7.1f}°  "
              f"conf={conf:.2f}{marker}")

    if write_previews:
        print(f"\nPreviews written to {PREVIEW_DIR}")


if __name__ == "__main__":
    main()
