# Omara Brand Assets

## ASCII Art Logo

`omara-ascii.txt` — The official Omara ASCII art logo. Used in:
- Boot splash (Plymouth theme)
- Terminal greetings
- Screensavers
- Documentation

## Rendering

To render as an image (e.g., for Plymouth watermark):

```bash
# Requires ImageMagick
convert -background none -fill '#ff5555' -font 'JetBrains-Mono' \
  -pointsize 48 -gravity center omara-ascii.txt watermark.png
```

### Auto-calculated Font Size

For 55% of screen width at 1920px (≈1056px):
- ASCII width: 60 characters
- Character width: ~10px at point size 48
- Calculated: `pointsize = (target_width / char_count) * 0.8`
- For 1024px image: `(1024 * 0.55 / 60) * 0.8 ≈ 75pt`

Actual font size auto-calculated per resolution.
