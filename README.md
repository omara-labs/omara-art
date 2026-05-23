# Omara Art

The official home for all visual and artistic assets for **Omara OS**.

This repository contains everything that defines how Omara *looks and feels* — from screensavers and animations to themes, palettes, graphics, and design systems.

## Scope

- Screensavers and idle visuals
- Lock screen and login themes
- Wallpapers (static + animated)
- Color palettes and theming tools
- Icon, cursor, and font assets
- Hyprland / compositor visual configurations
- Launcher, bar, and notification themes
- Boot splash and branding assets
- Any other artistic or visual elements

## Directory Structure

```
omara-art/
├── screensavers/     # Rust terminal screensavers (matrix, beams, bounce, etc.)
├── assets/
│   ├── palettes/     # Color palettes (base16, matugen, etc.)
│   ├── graphics/     # SVGs, logos, illustrations
│   ├── fonts/        # Font files and configurations
│   └── brand/        # Official branding assets
├── effects/          # Future visual effects and shaders
├── themes/           # GTK, Qt, Hyprland, terminal themes
├── wallpapers/
└── docs/
```

## Getting Started

### Screensavers

See [screensavers/README.md](screensavers/README.md) for build and usage instructions.

### Design Assets

Assets in `assets/` are intended for both the OS defaults and community contributors.  
Feel free to open issues or PRs for new palettes, graphics, or visual directions.

## Contributing

We welcome contributions in both code and design.

- Code: Rust screensavers/effects, build improvements, new visual tools
- Design: Color palettes, graphics, wallpapers, icon work, theme files

Please open an issue first if you're proposing something large.

## License

All code is licensed under the MIT License unless otherwise noted.  
Design assets are licensed under Creative Commons BY-SA 4.0 unless otherwise specified in their folder.

## Related Repositories

- [omara](https://github.com/omara-labs/omara) — Main OS repository
- [omara-config](https://github.com/omara-labs/omara-config) — Default configurations
- [omara-packages](https://github.com/omara-labs/omara-packages) — Packaging work

---

**Omara** — Beautiful, modern, and opinionated.
