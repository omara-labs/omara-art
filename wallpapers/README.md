# Omara Wallpapers

Pre-generated wallpapers for Omara OS, matching the theme system colors.

## Included Wallpapers

All wallpapers are 1920x1080 resolution with `#1e1e2e` background.

### Types
- **starfield** - Random starfield matching the bounce screensaver aesthetic
- **logo** - Omara ASCII logo centered on screen
- **gradient** - Vertical gradient from background to theme color

### Themes
- **red** - `#ff5555` (primary), `#ff0000` (bright)
- **blue** - `#5555ff` (primary), `#0000ff` (bright)
- **green** - `#55ff55` (primary), `#00ff00` (bright)

## Files

```
wallpapers/
├── starfield-red-1920x1080.png
├── starfield-blue-1920x1080.png
├── starfield-green-1920x1080.png
├── logo-red-1920x1080.png
├── logo-blue-1920x1080.png
├── logo-green-1920x1080.png
├── gradient-red-1920x1080.png
├── gradient-blue-1920x1080.png
└── gradient-green-1920x1080.png
```

## Usage

### With omara-cli
```bash
# List available wallpapers
omara wallpaper

# Set a wallpaper
omara wallpaper set starfield-red-1920x1080.png

# Cycle to next wallpaper
omara wallpaper next
```

### Manual
```bash
# Using swaybg (Niri/Wayland)
swaybg -i /path/to/omara-art/wallpapers/starfield-red-1920x1080.png

# Using feh (X11)
feh --bg-fill /path/to/omara-art/wallpapers/starfield-red-1920x1080.png
```

## Generating Custom Wallpapers

Run the generate script to create wallpapers:

```bash
cd wallpapers
bash generate.sh
```

This will generate all 9 wallpapers at 1920x1080 resolution.

### Customizing

Edit `generate.sh` to:
- Change resolution (modify `RESOLUTION` variable)
- Change colors (modify `RED`, `BLUE`, `GREEN` variables)
- Change star density (modify `star_count` calculation)
- Add new wallpaper types

## Requirements

- ImageMagick v7+ (`magick` command)
- `bc` for floating-point calculations
- JetBrains Mono font (for logo wallpapers)
