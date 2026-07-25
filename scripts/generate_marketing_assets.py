#!/usr/bin/env python3
"""Generate itch.io marketing assets for Rebellion."""

from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math
import os

# Try to load DejaVu Sans Mono, fallback to default
try:
    FONT_DIR = "/usr/share/fonts/truetype/dejavu/"
    font_title = ImageFont.truetype(FONT_DIR + "DejaVuSansMono-Bold.ttf", 82)
    font_subtitle = ImageFont.truetype(FONT_DIR + "DejaVuSansMono-Bold.ttf", 22)
    font_tagline = ImageFont.truetype(FONT_DIR + "DejaVuSansMono.ttf", 16)
    font_tiny = ImageFont.truetype(FONT_DIR + "DejaVuSansMono.ttf", 12)
    font_bg_large = ImageFont.truetype(FONT_DIR + "DejaVuSansMono-Bold.ttf", 120)
    font_bg_medium = ImageFont.truetype(FONT_DIR + "DejaVuSansMono-Bold.ttf", 28)
    font_bg_small = ImageFont.truetype(FONT_DIR + "DejaVuSansMono.ttf", 18)
except Exception as e:
    print(f"Font loading issue: {e}")
    font_title = ImageFont.load_default()
    font_subtitle = ImageFont.load_default()
    font_tagline = ImageFont.load_default()
    font_tiny = ImageFont.load_default()
    font_bg_large = ImageFont.load_default()
    font_bg_medium = ImageFont.load_default()
    font_bg_small = ImageFont.load_default()

# Color palette
colors = {
    "bg_dark": (5, 5, 8),
    "bg_panel": (10, 10, 18),
    "grid": (20, 25, 20),
    "green_bright": (170, 221, 170),
    "green_dim": (102, 170, 102),
    "green_faint": (40, 60, 40),
    "amber": (200, 170, 100),
    "white": (224, 224, 224),
    "red": (255, 100, 100),
}


def draw_scanlines(draw, width, height, spacing=4, alpha=30):
    """Draw subtle horizontal scan lines."""
    for y in range(0, height, spacing):
        draw.line([(0, y), (width, y)], fill=(*colors["bg_dark"], alpha), width=1)


def draw_grid(draw, width, height, spacing=40, color=None):
    """Draw a subtle grid pattern."""
    if color is None:
        color = colors["grid"]
    for x in range(0, width, spacing):
        draw.line([(x, 0), (x, height)], fill=color, width=1)
    for y in range(0, height, spacing):
        draw.line([(0, y), (width, y)], fill=color, width=1)


def draw_terminal_border(draw, x1, y1, x2, y2, color=None, width=2):
    """Draw a box-drawing border like a terminal panel."""
    if color is None:
        color = colors["green_dim"]
    # Draw corners with box-drawing chars isn't practical in PIL, so use lines
    draw.rectangle([x1, y1, x2, y2], outline=color, width=width)
    # Small corner accents
    corner_len = 10
    # Top-left
    draw.line([(x1, y1), (x1 + corner_len, y1)], fill=color, width=width + 1)
    draw.line([(x1, y1), (x1, y1 + corner_len)], fill=color, width=width + 1)
    # Top-right
    draw.line([(x2 - corner_len, y1), (x2, y1)], fill=color, width=width + 1)
    draw.line([(x2, y1), (x2, y1 + corner_len)], fill=color, width=width + 1)
    # Bottom-left
    draw.line([(x1, y2), (x1 + corner_len, y2)], fill=color, width=width + 1)
    draw.line([(x1, y2 - corner_len), (x1, y2)], fill=color, width=width + 1)
    # Bottom-right
    draw.line([(x2 - corner_len, y2), (x2, y2)], fill=color, width=width + 1)
    draw.line([(x2, y2 - corner_len), (x2, y2)], fill=color, width=width + 1)


def generate_cover_image():
    """630×500 cover image for itch.io project card."""
    width, height = 630, 500
    img = Image.new("RGB", (width, height), colors["bg_dark"])
    draw = ImageDraw.Draw(img)

    # Subtle gradient background
    for y in range(height):
        factor = y / height
        r = int(5 + factor * 8)
        g = int(5 + factor * 8)
        b = int(8 + factor * 10)
        draw.line([(0, y), (width, y)], fill=(r, g, b))

    # Grid
    draw_grid(draw, width, height, spacing=35, color=(15, 20, 15))

    # Main panel border
    margin = 25
    draw_terminal_border(draw, margin, margin, width - margin, height - margin,
                        color=colors["green_faint"], width=1)

    # Title: REBELLION
    title_text = "REBELLION"
    # Measure text for centering
    bbox = draw.textbbox((0, 0), title_text, font=font_title)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    tx = (width - tw) // 2
    ty = 140

    # Glow effect behind title
    for offset in range(15, 0, -3):
        alpha = 20 - offset
        glow_color = (40 + alpha, 80 + alpha, 40 + alpha)
        draw.text((tx, ty), title_text, font=font_title, fill=glow_color)

    # Main title
    draw.text((tx, ty), title_text, font=font_title, fill=colors["green_bright"])

    # Subtitle
    sub_text = "CONCORD CAPSULEER COMBAT ARCHIVE"
    bbox = draw.textbbox((0, 0), sub_text, font=font_subtitle)
    sw = bbox[2] - bbox[0]
    sx = (width - sw) // 2
    sy = ty + th + 20
    draw.text((sx, sy), sub_text, font=font_subtitle, fill=colors["amber"])

    # Tagline
    tag_text = "Historical Reconstruction Program  v2.1"
    bbox = draw.textbbox((0, 0), tag_text, font=font_tagline)
    tgw = bbox[2] - bbox[0]
    tgx = (width - tgw) // 2
    tgy = sy + 35
    draw.text((tgx, tgy), tag_text, font=font_tagline, fill=colors["green_dim"])

    # Bottom info bar
    bar_y = height - 55
    draw.line([(margin + 10, bar_y), (width - margin - 10, bar_y)],
              fill=colors["green_faint"], width=1)

    info_left = "SHOOTER  ·  ARCADE  ·  SCI-FI"
    draw.text((margin + 15, bar_y + 10), info_left, font=font_tiny, fill=colors["green_faint"])

    info_right = "OPEN SOURCE  ·  MIT"
    bbox = draw.textbbox((0, 0), info_right, font=font_tiny)
    irw = bbox[2] - bbox[0]
    draw.text((width - margin - 15 - irw, bar_y + 10), info_right, font=font_tiny, fill=colors["green_faint"])

    # Top header bar
    header_text = "YC 127  ·  AUTHENTICATED BATTLEFIELD TELEMETRY"
    draw.text((margin + 15, margin + 10), header_text, font=font_tiny, fill=colors["green_faint"])

    # Scanlines overlay
    overlay = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    overlay_draw = ImageDraw.Draw(overlay)
    for y in range(0, height, 3):
        overlay_draw.line([(0, y), (width, y)], fill=(0, 0, 0, 25))
    img = Image.alpha_composite(img.convert("RGBA"), overlay).convert("RGB")

    return img


def generate_background_image():
    """1920×1080 background for itch.io page."""
    width, height = 1920, 1080
    img = Image.new("RGB", (width, height), colors["bg_dark"])
    draw = ImageDraw.Draw(img)

    # Deep gradient
    for y in range(height):
        factor = y / height
        r = int(3 + factor * 5)
        g = int(3 + factor * 5)
        b = int(5 + factor * 8)
        draw.line([(0, y), (width, y)], fill=(r, g, b))

    # Large faint grid
    draw_grid(draw, width, height, spacing=60, color=(8, 12, 8))

    # Central panel area (faint)
    panel_margin_x = 200
    panel_margin_y = 100
    draw_terminal_border(draw, panel_margin_x, panel_margin_y,
                        width - panel_margin_x, height - panel_margin_y,
                        color=(15, 25, 15), width=1)

    # Large background text (watermark style)
    bg_text = "REBELLION"
    bbox = draw.textbbox((0, 0), bg_text, font=font_bg_large)
    btw = bbox[2] - bbox[0]
    bth = bbox[3] - bbox[1]
    btx = (width - btw) // 2
    bty = (height - bth) // 2 - 50

    # Very faint watermark
    for offset in range(20, 0, -4):
        alpha = int(8 - offset * 0.3)
        if alpha > 0:
            glow = (alpha, alpha * 2, alpha)
            draw.text((btx, bty), bg_text, font=font_bg_large, fill=glow)

    # Side text columns (like terminal data)
    left_col_x = 40
    right_col_x = width - 300
    col_y_start = 150
    line_height = 28

    left_lines = [
        ("ARCHIVE STATUS: ONLINE", colors["green_dim"]),
        ("NEURAL SYNC: STABLE", colors["green_dim"]),
        ("HISTORICAL ACCURACY: 99.84%", colors["green_dim"]),
        ("", colors["green_faint"]),
        ("ACTIVE SIMULATIONS:", colors["amber"]),
        ("  Caldari-Gallente War    [LOADED]", colors["green_dim"]),
        ("  Minmatar Rebellion      [STANDBY]", colors["green_faint"]),
        ("  Triglavian Invasion     [LOCKED]", colors["green_faint"]),
        ("", colors["green_faint"]),
        ("Factions Available:", colors["amber"]),
        ("  [✓] Caldari State", colors["green_dim"]),
        ("  [✓] Gallente Federation", colors["green_dim"]),
        ("  [ ] Minmatar Republic", colors["green_faint"]),
        ("  [ ] Amarr Empire", colors["green_faint"]),
    ]

    for i, (line, color) in enumerate(left_lines):
        draw.text((left_col_x, col_y_start + i * line_height), line, font=font_bg_small, fill=color)

    right_lines = [
        ("SYSTEM LOG:", colors["amber"]),
        ("YC 127.04.12  Loading archive...", colors["green_faint"]),
        ("YC 127.04.12  Verifying telemetry...", colors["green_faint"]),
        ("YC 127.04.12  Capsuleer interface primed.", colors["green_faint"]),
        ("", colors["green_faint"]),
        ("CONTROLS:", colors["amber"]),
        ("[WASD]  Movement", colors["green_faint"]),
        ("[SPACE] Fire", colors["green_faint"]),
        ("[SHIFT] Ability", colors["green_faint"]),
        ("", colors["green_faint"]),
        ("MISSION PARAMS:", colors["amber"]),
        ("Duration: 20-30 min", colors["green_faint"]),
        ("Difficulty: Variable", colors["green_faint"]),
        ("Objectives: Survive", colors["green_faint"]),
    ]

    for i, (line, color) in enumerate(right_lines):
        draw.text((right_col_x, col_y_start + i * line_height), line, font=font_bg_small, fill=color)

    # Bottom status bar
    bar_height = 40
    bar_y = height - bar_height
    draw.rectangle([(0, bar_y), (width, height)], fill=(8, 12, 8))
    draw.line([(0, bar_y), (width, bar_y)], fill=colors["green_faint"], width=1)

    status_text = "CONCORD CAPSULEER COMBAT ARCHIVE  ·  HISTORICAL RECONSTRUCTION PROGRAM  ·  VERSION 2.1"
    draw.text((40, bar_y + 12), status_text, font=font_bg_small, fill=colors["green_faint"])

    # Top header
    draw.rectangle([(0, 0), (width, 30)], fill=(8, 12, 8))
    draw.line([(0, 30), (width, 30)], fill=colors["green_faint"], width=1)
    header = "ARCHIVE TERMINAL  ·  SESSION: LIVE  ·  AUTHENTICATED"
    draw.text((40, 8), header, font=font_bg_small, fill=colors["green_faint"])

    # Scanlines
    overlay = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    overlay_draw = ImageDraw.Draw(overlay)
    for y in range(0, height, 4):
        overlay_draw.line([(0, y), (width, y)], fill=(0, 0, 0, 15))
    img = Image.alpha_composite(img.convert("RGBA"), overlay).convert("RGB")

    return img


def main():
    out_dir = "assets/marketing"
    os.makedirs(out_dir, exist_ok=True)

    print("Generating cover image (630×500)...")
    cover = generate_cover_image()
    cover_path = os.path.join(out_dir, "itch-cover-630x500.png")
    cover.save(cover_path, "PNG")
    print(f"  Saved: {cover_path}")

    print("Generating background image (1920×1080)...")
    bg = generate_background_image()
    bg_path = os.path.join(out_dir, "itch-background-1920x1080.png")
    bg.save(bg_path, "PNG")
    print(f"  Saved: {bg_path}")

    print("\nDone. Upload these through the itch.io project settings page.")


if __name__ == "__main__":
    main()
