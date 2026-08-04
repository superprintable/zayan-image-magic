# Zayan Image Magic

Local desktop image converter by **Syed Faraz Ahmad**. Convert PNG, JPEG, WebP, AVIF, GIF, BMP, and TIFF entirely on your machine — no uploads.

**Repo:** [github.com/superprintable/zayan-image-magic](https://github.com/superprintable/zayan-image-magic)  
**Latest release:** [v0.1.0](https://github.com/superprintable/zayan-image-magic/releases/tag/v0.1.0)

## Download (Windows x64)

Grab installers from the [Releases](https://github.com/superprintable/zayan-image-magic/releases/latest) page:

| File | Use |
|------|-----|
| **Zayan-Image-Magic-Setup-0.1.0-x64.exe** | Recommended installer (NSIS) — Start Menu + uninstall |
| **Zayan-Image-Magic-0.1.0-x64.msi** | Enterprise / msiexec installer |
| **Zayan Image Magic.exe** | Portable exe (double-click; needs WebView2) |

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
- [NASM](https://www.nasm.us/) on `PATH` (needed to build AVIF via rav1e). Optional helper path: put binaries in `tools/nasm/`

### Setup

```bash
git clone https://github.com/superprintable/zayan-image-magic.git
cd zayan-image-magic
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

## Accuracy notes

- **Lossless** targets (PNG, WebP lossless, AVIF quality 100, BMP, TIFF) keep pixel fidelity within the codec’s lossless path.
- **Lossy** targets (JPEG, lossy WebP/AVIF, GIF) cannot be bit-identical; the app defaults to quality 90.

## Stack

- Tauri 2 + React + TypeScript (Windows NSIS + MSI bundling)
- Rust `image` + `webp` + `ravif` conversion engine

Made by Syed Faraz Ahmad · 2026
