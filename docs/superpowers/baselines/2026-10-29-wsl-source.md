# WSL source and firmware baseline — 2026-10-29

Environment: WSL2. These results do not validate Windows Tauri, USB, audio, installers, or physical hardware.

## Toolchain

Recorded before installing project dependencies.

- node: v22.21.0
- npm: 10.9.4
- python3: Python 3.12.3
- git: git version 2.43.0
- cc: cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0
- rustc: absent on WSL; Windows owns Rust
- cargo: absent on WSL; Windows owns Cargo
- platformio: absent until Task 4 (installed later as PlatformIO Core 6.1.19 in a venv outside the repository)
- rustc: installed later from the Ubuntu package, 1.75.0, for the firmware media codec test harness only

## Results

- source safety: pass (exit 0)
- npm test: pass (exit 0)
- npm run test:bridge: pass (exit 0)
- npm run build:web: pass (exit 0)
- web preview: pass (plan check revised: page renders with no Tauri runtime, shows board-dev-direct-001 and USB 未连接)
- pytest: pass (222 passed, 1 skipped, exit 0). The first run had 10 errors in firmware/tests/p4_media_codec_test.py because rustc was absent on WSL. The user installed rustc 1.75.0 from the Ubuntu package (not the stable channel), and the rerun passed.
- protocol contract: pass (62 ok lines, exit 0)
- builtin bundle: pass (--check exit 0, no diff to tracked files)
- PlatformIO: PlatformIO Core, version 6.1.19
- v1 image: pass (1, 199) {"appBytes": 2342496, "otaBytes": 2621440, "otaHeadroomBytes": 278944, "internalStaticBytes": 135980, "externalStaticBytes": 1572004, "minimumOtaHeadroomBytes": 131072, "maximumInternalStaticBytes": 393216}
- v3 image: pass (300, 399) {"appBytes": 2357984, "otaBytes": 2621440, "otaHeadroomBytes": 263456, "internalStaticBytes": 340560, "externalStaticBytes": 1572004, "minimumOtaHeadroomBytes": 131072, "maximumInternalStaticBytes": 393216}
- dependency locks: pass (normalizer ran, no diff, no machine paths)

## Status

WSL source checks, the web preview, firmware host tests, and sequential PlatformIO 6.1.19 builds for v1 and v3 passed. The project is source-reproduced and firmware-built. It is not hardware-validated. Windows Tauri, Claude Code lifecycle observation, and device flashing remain separate work.

Warnings noted for later review, not blocking: Vite import and chunk-size warnings during `build:web`; `npm ci` audit notice (2 high); one skipped Node test; the v1 and v3 builds warn about a 4104-byte `parse_bounded_scene` frame, and v3 also warns about a 4240-byte LVGL `lv_tjpgd.c` frame and a linker `sram_seg` memory-type notice. Budgets and checker still passed.
