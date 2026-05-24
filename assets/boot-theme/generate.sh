#!/usr/bin/env bash
# Generate Omara Boot Theme Assets
# Requires: ImageMagick

set -euo pipefail

THEME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$THEME_DIR")/brand"
OUTPUT_DIR="/usr/share/plymouth/themes/omara-boot"
RESOLUTION="1024x1024"
FONT="JetBrains-Mono"

# =============================================================================
# CONFIGURATION
# =============================================================================

# Colors (from omara-core theme)
RED="#ff5555"
RED_BRIGHT="#ff0000"
BACKGROUND="#00000000"  # Transparent

# ASCII art file
ASCII_FILE="$SOURCE_DIR/omara-ascii.txt"

# Output files
WATERMARK="$OUTPUT_DIR/watermark.png"

# =============================================================================
# FUNCTIONS
# =============================================================================

# Auto-calculate font size for 55% of image width
calculate_font_size() {
    local img_width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local target_width=$(echo "$img_width * 0.55" | bc)
    local char_count=60  # Approximate width of ASCII art
    local char_width_px=10  # Average monospace char width at pointsize 48
    
    # Scale factor: target_width / (char_count * char_width_at_48) * 48
    # char_width_at_48 ≈ 10px, so: font_size = target_width / char_count * 0.8 * 48
    echo "$(echo "scale=0; $target_width / $char_count * 0.8 * 48" | bc)" | cut -d'.' -f1
}

# Generate watermark
generate_watermark() {
    local fontsize=$(calculate_font_size)
    echo "🎨 Generating watermark (font size: $fontsize)..."
    
    convert \
        -background "$BACKGROUND" \
        -fill "$RED" \
        -font "$FONT" \
        -pointsize "$fontsize" \
        -gravity center \
        -interline-spacing 10 \
        "$ASCII_FILE" \
        "$WATERMARK"
}

# Generate throbber frames (dots animation)
generate_throbber() {
    local frames=11
    local size=128
    local dot_size=24
    local spacing=20
    local dot_color="$RED"
    local active_dot_color="$RED_BRIGHT"
    local y_offset=48  # Vertically center between OMARA and bottom
    
    echo "🎬 Generating $frames throbber frames..."
    
    # Patterns: O●oooo, oO●ooo, ooO●oo, oooO●o, ooooO●, ooooO●, oooO●o, ooo●oo, oo●ooo, o●oooo, ●ooooo
    local patterns=(
        "●ooooo"
        "o●oooo"
        "oo●ooo"
        "ooo●oo"
        "oooo●o"
        "ooooo●"
        "oooo●o"
        "ooo●oo"
        "oo●ooo"
        "o●oooo"
        "●ooooo"
    )
    
    for i in $(seq 0 $((frames - 1))); do
        local pattern="${patterns[$i]}"
        local filename="$OUTPUT_DIR/throbber-$(printf '%04d' $((i+1))).png"
        
        # Create transparent canvas
        convert -size "${size}x${size}" xc:none "$filename"
        
        # Calculate x positions for dots
        local total_width=$(( ${#pattern} * (dot_size + spacing) - spacing ))
        local start_x=$(( (size - total_width) / 2 ))
        
        for j in $(seq 0 $(( ${#pattern} - 1 ))); do
            local char="${pattern:$j:1}"
            local x=$(( start_x + j * (dot_size + spacing) ))
            local y=$(( (size - y_offset) / 2 ))
            
            if [[ "$char" == "●" ]]; then
                color="$active_dot_color"
            else
                color="$dot_color"
            fi
            
            # Draw circle
            convert "$filename" \
                -fill "$color" \
                -draw "circle $x,$y $((x + dot_size/2)),$((y + dot_size/2))" \
                "$filename"
        done
    done
}

# =============================================================================
# MAIN
# =============================================================================

echo "🚀 Generating Omara Boot Theme Assets"
echo "=================================="

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate assets
generate_watermark
generate_throbber

echo ""
echo "✅ Done! Assets generated in $OUTPUT_DIR"
echo ""
echo "To install:"
echo "  sudo cp -r $OUTPUT_DIR/* /usr/share/plymouth/themes/omara-boot/"
echo "  sudo plymouth-set-default-theme omara-boot"
