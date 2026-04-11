# Task #109: Windowed Browser Binary Entry Point (browser_main.rs)

State: PROPOSED
Created: 2026-04-11T15:10:57.750000+00:00

# Windowed Browser Binary Entry Point

## What & Why
The browser rendering pipeline (parser, layout engine, CPU renderer, tab manager, PlenumColor mesh, script executor) is fully built in `src/kernel/src/browser/` and works against a `CpuFramebuffer`. But it only runs inside the bare-metal kernel (`#![no_std]`, `#![no_main]`), which boots as a standalone OS — not a Windows application.

The MSI installer tooling (`tools/plenum-pack/`) and Plenum Launcher (`tools/plenum-launcher/`) are both production-ready, but the browser has no Windows `.exe` to install because the only binary target is the bare-metal kernel.

This task creates a userspace windowed entry point that reuses the exact same browser modules with a `winit` window underneath instead of raw hardware framebuffer access. The rendering code, layout engine, parser, tabs, color pipeline — all unchanged. Just a different front door.

## Done looks like
- `src/kernel/src/browser_main.rs` exists with a `fn main()` that opens a `winit` window and runs the browser pipeline
- A `windowed` feature is added to `src/kernel/Cargo.toml` that pulls in `winit` and `raw-window-handle`
- A `plenumnet-browser` binary target is added with `required-features = ["std", "browser-crates", "windowed"]`
- `CpuFramebuffer::pixels()` data is blitted to the `winit` window surface each frame using `softbuffer` (or equivalent CPU-to-window blit crate)
- The browser initializes identically to `main.rs` lines 380-418: creates a `Distributor`, creates `Browser::new(width, height, distributor)`, opens `plenum://home`, renders via `render_home_page()`, applies mesh color
- Event loop handles window resize, keyboard/mouse input forwarded to `BrowserInputHandler`, close/minimize
- Builds with `cargo build --release --features browser-crates,windowed --target aarch64-pc-windows-msvc` producing `plenumnet-browser.exe`
- A `plenum-app.toml` manifest is created for the browser so `plenum-pack` can package it into an MSI

## Out of scope
- GPU rendering path (Phase 2 — separate milestone)
- Changes to the existing browser modules (parser, layout, renderer, tabs, color, script)
- Changes to the bare-metal kernel entry point (`main.rs`)
- Changes to `plenum-pack` or WiX templates (they already handle arbitrary binaries via manifest)
- Changes to the Plenum Launcher (it discovers apps via registry automatically)

## Tasks
1. Add `windowed` feature to `Cargo.toml` gating `winit` and `softbuffer` dependencies. Add the `plenumnet-browser` binary target with `required-features = ["std", "browser-crates", "windowed"]`.
2. Create `src/kernel/src/browser_main.rs` — a `fn main()` (with std) that initializes `winit` event loop, creates a window, instantiates the `Browser` the same way `kernel_main` does, and enters a render loop that blits `CpuFramebuffer::pixels()` to the window surface via `softbuffer`.
3. Wire input events — forward `winit` keyboard/mouse events to `BrowserInputHandler` so tab navigation, scrolling, and text input work.
4. Create `src/kernel/plenum-app.toml` for the browser binary so `plenum-pack` can generate an MSI from it.

## Relevant files
- `src/kernel/Cargo.toml`
- `src/kernel/src/main.rs:370-450`
- `src/kernel/src/browser/mod.rs:41-80`
- `src/kernel/src/browser/render_cpu.rs:59-93`
- `src/kernel/src/browser/input.rs`
- `tools/plenum-launcher/plenum-app.toml`
- `tools/plenum-pack/src/manifest.rs:8-58`
