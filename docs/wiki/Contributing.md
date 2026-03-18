# Contributing

## Development Setup

### Prerequisites
- **Rust** (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** 22+: via your package manager or [nodejs.org](https://nodejs.org)
- **Platform libraries**:
  - Arch Linux: `sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3`
  - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - macOS: `xcode-select --install`

### Getting Started

```bash
git clone https://github.com/your-org/macenv.git
cd macenv
npm install
npm run tauri dev
```

This starts the Vite dev server (port 1420) and builds/launches the Tauri app with hot reload.

## Project Structure

See [Architecture](Architecture.md) for the full directory layout.

Key areas:
- `src/` — Svelte 5 frontend (components, stores, CSS)
- `src-tauri/src/` — Rust backend (Tauri commands, services, package management)
- `src-tauri/resources/packages.toml` — Package name mappings
- `.github/workflows/build.yml` — CI/CD pipeline

## Code Style

### Rust
- Format with `rustfmt` (standard config)
- Lint with `cargo clippy`
- Use `thiserror` for error types
- Use `async-trait` for async trait methods
- All async operations use `tokio`

### Svelte/JavaScript
- Svelte 5 with runes (`$state`, `$derived`, `$effect`, `$props()`)
- Store files must use `.svelte.js` extension for runes
- No Tailwind — use CSS variables from `app.css`
- No TypeScript (for now — may be added later)

### CSS
- Use CSS custom properties from the design system
- Scoped styles in `.svelte` files via `<style>` blocks
- Global styles only in `app.css`
- Follow the naming convention: `.component-name__element--modifier`

## Adding a New Service

To add support for a new service (e.g., MongoDB):

1. **Add to `packages.toml`**:
   ```toml
   [mongodb]
   description = "MongoDB document database"
   category = "database"
   has_service = true
   versioned = true

   [mongodb.managers.homebrew]
   pattern = "mongodb-community@{version}"
   service_name = "mongodb-community@{version}"
   tap = "mongodb/brew"
   ```

2. **Add to service lists** in `brew_services.rs`, `systemd.rs`

3. **Create a ServiceCard** entry (automatic from `get_services`)

## Adding a New Package Manager

1. Create `src-tauri/src/package_manager/new_manager.rs`
2. Implement the `PackageManager` trait
3. Add `pub mod new_manager;` in `package_manager/mod.rs`
4. Update `create_package_manager()` detection logic
5. Add mappings for all packages in `packages.toml`

## Pull Request Guidelines

1. Fork and create a feature branch
2. Make your changes with clear commit messages
3. Ensure `cargo clippy` passes with no warnings
4. Test on your platform (`npm run tauri dev`)
5. Open a PR against `main`

## Reporting Issues

Please include:
- OS and version
- MacEnv version
- Steps to reproduce
- Expected vs actual behavior
- Relevant log output

## License

All contributions are under the MIT license.
