# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

RanranToolkit is a Tauri 2 desktop app for Android device management, built with a Vue 3 frontend and a Rust backend. The app manages ADB/Fastboot/Scrcpy/Aria2/link-dumper runtime tools, Android device state, downloads, ROM data, app management, file management, boot image patching, and update/runtime bootstrapping.

## Common commands

This project uses Yarn and has a `bin` git submodule for runtime tools.

```bash
# install frontend dependencies
yarn install

# initialize runtime-tool submodule after a fresh clone
git submodule update --init --recursive

# run the full Tauri desktop app in development
yarn tauri dev

# run only the Vite frontend dev server on port 1420
yarn dev

# build only the frontend into dist/
yarn build

# build Tauri release bundles after cleaning Rust target output
yarn build:tauri

# clean Tauri/Rust build output
yarn clean:tauri

# build release bundles, portable zip, and runtime split archives
yarn build:all

# generate bin/rom-data/codename-model-map.json for development
yarn dev:generate-codename-model-map

# build only the portable bootstrap zip after a Tauri release build
yarn build:portable

# build only runtime split archives and runtime-manifest.json from bin/
yarn build:runtime-assets
```

There are currently no configured npm/yarn lint or test scripts. For Rust-only checks, run Cargo directly against the Tauri manifest:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Release/runtime notes

- `bin/` is a git submodule (`ranran-toolkit-bin`) containing Android/runtime tools. A fresh clone needs `git submodule update --init --recursive`.
- Release packages no longer bundle the full `bin/` directory. On first launch, the app prepares runtime assets from split archives described by `runtime-manifest.json`.
- `yarn build:runtime-assets` writes split runtime archives to `bin/cloud-parts/`; formal releases must publish the app bundle plus the generated `runtime-manifest.json` and all `bin-runtime.zip.*` parts.
- The runtime manifest URL defaults in `src-tauri/src/commands/runtime_assets.rs` and can be overridden at compile time with `RANRAN_RUNTIME_MANIFEST_URL`.
- The Node runtime packer supports `RUNTIME_BASE_URL` when generating `runtime-manifest.json`.

## High-level architecture

### Frontend

- Entry point: `src/main.js` creates the Vue app, installs Element Plus with Chinese locale, and mounts `src/App.vue`.
- `src/App.vue` first calls the Rust `prepare_runtime_assets` command and listens for `runtime-assets-progress`. Routed UI is rendered only after runtime tools are ready; it then calls `warmup_platform_tools` and initializes global Scrcpy auto-mirroring.
- Routing is centralized in `src/router/index.js` with hash history. `src/components/layout/Sidebar.vue` builds the navigation menu directly from `router.options.routes`, so route metadata controls the sidebar label/icon/color.
- The top-bar refresh behavior calls `refresh()` on the currently routed component if that page exposes it with `defineExpose({ refresh })`.
- Shared reactive state is implemented with module-level Vue refs rather than Pinia stores. Key stores are `src/utils/deviceStore.js`, `src/utils/scrcpyStore.js`, `src/utils/themeStore.js`, and `src/utils/updateStore.js`.
- Device polling starts from `src/components/layout/StatusBar.vue`, via `useDeviceStore().startPolling()`. It polls `get_connected_devices` about every 600 ms and keeps a short offline grace window to avoid flicker.
- Frontend API wrappers live in `src/api/*.js`. Most device-related wrappers in `src/api/device.js` automatically inject the currently selected serial from `deviceStore` before invoking Rust commands.
- Page implementations live under `src/views/<feature>/index.vue`; feature-specific child components are grouped in sibling `components/` directories and often re-exported through `components/index.js`.
- Styling is SCSS-based. Global entry is `src/assets/main.scss`, which imports Element Plus dark CSS variables plus local variable/base partials. Many page/component styles are scoped SCSS using the same CSS custom properties.

### Rust/Tauri backend

- Tauri entry point is `src-tauri/src/main.rs`; application setup and command registration are in `src-tauri/src/lib.rs`.
- `lib.rs` registers Tauri plugins (`fs`, `opener`, `dialog`), initializes `AppPaths`, process tracking, `ExitCleanupState`, and a shared `DownloadManager`, then registers all IPC commands with `tauri::generate_handler!`.
- Rust commands are grouped by feature under `src-tauri/src/commands/`: device/ADB operations, apps, file manager, downloader, payload extraction, boot patching, ROM providers, runtime assets, and system utilities.
- `src-tauri/src/adb/core.rs` resolves tool paths and wraps ADB/Fastboot/Scrcpy execution. In dev builds it uses the repository `bin/`; in non-dev builds it uses the per-user app-local runtime cache under `runtime/bin`.
- Fastboot commands are serialized through a global async lock/cache in `adb/core.rs`; keep that pattern for new Fastboot operations rather than spawning unconstrained Fastboot commands.
- `src-tauri/src/utils/process.rs` tracks spawned helper processes by kind so Scrcpy, ADB server/client, Aria2, link-dumper, and related children can be cleaned up on exit.
- `src-tauri/src/commands/runtime_assets.rs` owns first-launch runtime preparation: manifest fetch, local cache validation, split-part download, SHA-256 checks, archive extraction, and install-state writing.
- `src-tauri/src/commands/downloader.rs` uses Aria2, emits `download-progress` events, and stores task state in the managed `DownloadManager`.
- `src-tauri/tauri.conf.json` defines a hidden undecorated main window plus a splashscreen; Vite dev URL is fixed to `http://127.0.0.1:1420`.

## IPC and event conventions

- Frontend-to-backend calls use `invoke('<snake_case_command>', payload)` from `@tauri-apps/api/core`.
- Command names must be registered in `src-tauri/src/lib.rs` and implemented with `#[tauri::command]` in the relevant Rust command module.
- Backend-to-frontend progress uses Tauri events. Existing important event names are `runtime-assets-progress`, `download-progress`, and `scrcpy-exited`.
- When adding device-scoped frontend APIs, follow `src/api/device.js`: resolve `selectedSerial` in the wrapper and pass `serial` to Rust, so pages do not each duplicate serial-selection logic.

## UI patterns to preserve

- Keep route-level pages as the owner of feature state and data loading; keep presentational pieces in that feature's `components/` folder.
- If a page should support the global refresh button, expose a `refresh` function from the page component.
- Use existing Element Plus components, `SmartIcon`, `FloatingLog`, and the shared CSS variables before introducing new UI primitives.
- Several long-running flows are event-driven; remember to unregister Tauri event listeners in `onUnmounted`/`onBeforeUnmount`.

## Files and generated output

- Generated/build outputs are ignored: `dist/`, `src-tauri/target/`, `src-tauri/gen/`, `.runtime-build/`, and `src-tauri/Cargo.lock`.
- `src-tauri/examples/generate_codename_model_map.rs` is invoked by `yarn dev:generate-codename-model-map` and writes `bin/rom-data/codename-model-map.json`.
- `scripts/build-zip.cjs` expects `src-tauri/target/release/RanranToolkit.exe` to already exist.
- `scripts/build-runtime-assets.cjs` expects `bin/` to exist and excludes `bin/cloud-parts/` from the generated runtime archive.
  ""
