#!/usr/bin/env bash
# Generate Omara Wallpapers
# Requires: ImageMagick v7+, bc

set -euo pipefail

WALLPAPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BRAND_DIR="$WALLPAPER_DIR/../assets/brand"

# Default resolution
RESOLUTION="1920x1080"

# Theme colors
RED="#ff5555"
RED_BRIGHT="#ff0000"
BLUE="#5555ff"
BLUE_BRIGHT="#0000ff"
GREEN="#55ff55"
GREEN_BRIGHT="#00ff00"
BACKGROUND="#1e1e2e"

# Font
FONT="JetBrains-Mono"

# =============================================================================
# WALLPAPER GENERATORS
# =============================================================================

generate_starfield() {
    local theme="$1"
    local color="$2"
    local filename="${WALLPAPER_DIR}/starfield-${theme}-${RESOLUTION}.png"
    
    local width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local height=$(echo "$RESOLUTION" | cut -d'x' -f2)
    local star_count=$((width * height / 800))
    
    echo "  Generating starfield ($theme, $star_count stars)..."
    
    # Create a temporary file with all star draw commands
    local tmpfile=$(mktemp)
    
    for i in $(seq 1 $star_count); do
        local x=$((RANDOM % width))
        local y=$((RANDOM % height))
        local opacity=$((RANDOM % 60 + 40))
        local star_size=$((RANDOM % 3 + 1))
        
        local r=$(printf "%d" "0x${color:1:2}")
        local g=$(printf "%d" "0x${color:3:2}")
        local b=$(printf "%d" "0x${color:5:2}")
        local alpha=$(echo "scale=2; $opacity / 100" | bc)
        
        echo "fill rgba($r,$g,$b,$alpha) circle $x,$y $((x + star_size)),$y" >> "$tmpfile"
    done
    
    # Single magick call with all draw commands
    magick -size "$RESOLUTION" xc:"$BACKGROUND" \
        -draw @"$tmpfile" \
        "$filename"
    
    rm -f "$tmpfile"
}

generate_logo() {
    local theme="$1"
    local color="$2"
    local filename="${WALLPAPER_DIR}/logo-${theme}-${RESOLUTION}.png"
    
    local width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local font_size=$((width * 50 / 100 / 6))
    
    echo "  Generating logo ($theme)..."
    
    magick -size "$RESOLUTION" xc:"$BACKGROUND" "$filename"
    
    magick "$filename" \
        -font "$FONT" \
        -pointsize "$font_size" \
        -fill "$color" \
        -gravity center \
        -interline-spacing 40 \
        -annotate +0+0 "@${BRAND_DIR}/omara-ascii.txt" \
        "$filename"
}

generate_gradient() {
    local theme="$1"
    local color="$2"
    local filename="${WALLPAPER_DIR}/gradient-${theme}-${RESOLUTION}.png"
    
    echo "  Generating gradient ($theme)..."
    
    magick -size "$RESOLUTION" \
        -define gradient:direction=vertical \
        gradient:"$BACKGROUND-$color" \
        "$filename"
}

# =============================================================================
# MAIN
# =============================================================================

echo "🎨 Generating Omara Wallpapers (${RESOLUTION})"
echo "============================================"

# Red theme
generate_starfield "red" "$RED"
generate_logo "red" "$RED"
generate_gradient "red" "$RED"

# Blue theme
generate_starfield "blue" "$BLUE"
generate_logo "blue" "$BLUE"
generate_gradient "blue" "$BLUE"

# Green theme
generate_starfield "green" "$GREEN"
generate_logo "green" "$GREEN"
generate_gradient "green" "$GREEN"

echo ""
echo "✅ Wallpapers generated in $WALLPAPER_DIR"
echo ""
echo "Files:"
ls -1 "${WALLPAPER_DIR}/"*.png 2>/dev/null | while read f; do
    echo "  - $(basename "$f")"
done
