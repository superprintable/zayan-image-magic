# Zayan Image Magic

Local desktop image converter by **Syed Faraz Ahmad**. Convert PNG, JPEG, WebP, AVIF, GIF, BMP, and TIFF entirely on your machine — no uploads.

## Download (Windows x64)

Ready-to-run builds are in the [`release/`](release/) folder:

| File | Use |
|------|-----|
| **Zayan-Image-Magic-Setup-0.1.0-x64.exe** | Recommended installer (NSIS) — Start Menu + uninstall |
| **Zayan-Image-Magic-0.1.0-x64.msi** | Enterprise / msiexec installer |
| **Zayan Image Magic.exe** | Portable exe (double-click; needs WebView2) |

Double-click the **Setup** exe to install for yourself, or run the portable **Zayan Image Magic.exe** with no install step.

Requires [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on most Windows 10/11 PCs).

## Features

- Drag-and-drop or browse multi-file queues
- Destination formats: WebP, AVIF, JPEG, PNG, GIF, BMP, TIFF
- Quality slider + lossless mode where supported
- EXIF orientation applied on decode
- Alpha preserved when the target supports it; flattened onto white for JPEG/BMP
- Batch convert with per-file error isolation

## Develop from source

### Requirements

- Windows 10/11
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable)
- NASM is bundled under `tools/nasm` (used when building AVIF)

### Setup

```bash
npm install
```

If `npm install` fails with a certificate error:

```bash
set NODE_OPTIONS=--use-system-ca
npm install
```

### Dev

```bash
npm run tauri:dev
```

### Build Windows installers (.exe / .msi)

```bash
npm run tauri:build
```

Outputs:

- `src-tauri/target/release/zayan-image-magic.exe` — raw binary
- `src-tauri/target/release/bundle/nsis/*-setup.exe` — NSIS installer
- `src-tauri/target/release/bundle/msi/*.msi` — MSI installer

Friendly copies are also written to `release/` after a successful build (or copy them from the paths above).

## Accuracy notes

- **Lossless** targets (PNG, WebP lossless, AVIF quality 100, BMP, TIFF) keep pixel fidelity within the codec’s lossless path.
- **Lossy** targets (JPEG, lossy WebP/AVIF, GIF) cannot be bit-identical; the app defaults to quality 90.

## Stack

- Tauri 2 + React + TypeScript (2026 Windows NSIS + MSI bundling)
- Rust `image` + `webp` + `ravif` conversion engine

Made by Syed Faraz Ahmad · 2026
