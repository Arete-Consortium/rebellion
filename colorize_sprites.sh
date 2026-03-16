#!/bin/bash
# Colorize ship sprites with faction colors using ImageMagick
# Preserves original format while tinting

cd "$(dirname "$0")/assets/ships"

# Faction color tints (hue rotation in degrees)
# Amarr: Gold/bronze - warm yellow-orange
# Caldari: Blue-gray - cool blue
# Gallente: Olive-green - green tint
# Minmatar: Rust/copper - red-orange

colorize_amarr() {
    # Gold tint: multiply with gold color
    convert "$1" \( +clone -fill "rgb(220,180,80)" -colorize 40% \) -compose Multiply -composite "$1"
}

colorize_caldari() {
    # Blue-gray tint
    convert "$1" \( +clone -fill "rgb(140,160,200)" -colorize 30% \) -compose Multiply -composite "$1"
}

colorize_gallente() {
    # Olive-green tint
    convert "$1" \( +clone -fill "rgb(140,180,120)" -colorize 35% \) -compose Multiply -composite "$1"
}

colorize_minmatar() {
    # Rust/copper tint
    convert "$1" \( +clone -fill "rgb(200,140,100)" -colorize 40% \) -compose Multiply -composite "$1"
}

colorize_triglavian() {
    # Red tint
    convert "$1" \( +clone -fill "rgb(200,100,100)" -colorize 45% \) -compose Multiply -composite "$1"
}

colorize_pirate() {
    # Dark gunmetal - slight desaturation and darken
    convert "$1" -modulate 90,70,100 "$1"
}

echo "Colorizing ship sprites..."

# Amarr ships
for id in 589 593 594 597 598 624 625 626 643 11184 11186 16240 24690; do
    [ -f "${id}.png" ] && colorize_amarr "${id}.png" && echo "  Amarr: ${id}"
done

# Caldari ships
for id in 583 602 603 605 608 638 639 640 16236 16238 24688 35683; do
    [ -f "${id}.png" ] && colorize_caldari "${id}.png" && echo "  Caldari: ${id}"
done

# Gallente ships
for id in 585 586 587 591 592 630 641 645 16242 24696; do
    [ -f "${id}.png" ] && colorize_gallente "${id}.png" && echo "  Gallente: ${id}"
done

# Minmatar ships
for id in 11371 11373 11377 11381 11387 11400 11547 11566 11567 24694 24700 35685; do
    [ -f "${id}.png" ] && colorize_minmatar "${id}.png" && echo "  Minmatar: ${id}"
done

# Triglavian
for id in 47269 47466; do
    [ -f "${id}.png" ] && colorize_triglavian "${id}.png" && echo "  Triglavian: ${id}"
done

# Pirate/special
for id in 1944 3764 20185 23757 23911 23915 24483; do
    [ -f "${id}.png" ] && colorize_pirate "${id}.png" && echo "  Pirate: ${id}"
done

echo "Done!"
