# HachimoDock Reproduction Baseline Design

## Purpose

Establish a repeatable, non-commercial development baseline for HachimoDock before changing product behavior or purchasing a complete hardware build. The first supported Agent path is Claude Code.

The work proceeds from source verification to native desktop verification and then to a minimal hardware prototype. Results from WSL, Windows, and physical hardware are recorded separately so that one environment is not treated as evidence for another.

## Repository and Branch

- Working directory: `~/sourcecode/HachimoDock`
- Upstream remote: `https://github.com/YizhengWw/HachimoDock.git`
- Base branch: the freshly cloned current `main`
- Local work branch: `work/reproduction-baseline`
- Do not reuse clones made before the upstream history cleanup on 2026-10-28.
- Do not commit API keys, certificates, private runtime binaries, machine-specific proxy settings, or generated build output.
- Configure a personal fork only when work is ready to be published; until then, `origin` remains the upstream repository.

## License Boundary

This baseline is for personal learning, modification, and non-commercial desktop use. The repository's HachimoDock Project License is source-available and is not an OSI open-source license. Sale, paid services, commercial integration, preinstallation for sale, or independent products require prior written permission from the copyright holder.

## Environment Responsibilities

### WSL2

WSL2 is the primary source-development environment and owns:

- Node.js and Bridge tests
- React/Vite web development and production web builds
- Python firmware host tests
- Built-in component resource validation
- PlatformIO firmware builds for both supported ESP32-P4 chip families
- Image, manifest, and budget inspection

The repository stays in the WSL ext4 filesystem. It is not placed under `/mnt/c`, where package installation and firmware builds are more vulnerable to slow small-file access, path semantics, and permission differences.

### Windows host

Windows owns:

- Rust stable and Cargo
- MSVC C++ Build Tools and WebView2
- Native Tauri development and Windows packaging
- Real Claude Code detection, Hook installation, and lifecycle observation
- Windows UI Automation behavior
- Serial flashing and device communication

WSL results do not count as native Windows desktop, USB, audio, or installer validation.

## System Boundaries and Data Flow

### Web and desktop UI

React and Vite provide the management UI. `npm run dev:web` is a UI-only preview and does not provide Tauri IPC, USB, system permissions, local Agent detection, or managed Bridge behavior.

The native application runs through Tauri. React invokes Rust commands for system capabilities rather than accessing serial devices or Agent files directly.

### Claude Code status path

The baseline status path is:

```text
Claude Hook and Claude JSONL transcript
  -> Node bridge on loopback port 23333
  -> Agent Session Bus on loopback port 8181
  -> React session UI and Rust state forwarder
  -> ESP32-P4 over USB after hardware is introduced
```

Both loopback services must be healthy before the Bridge is considered ready. Hook installation must merge with existing Claude settings rather than replacing them. Unknown permission policies never default to approval.

The first phase verifies read-only lifecycle observation: session start, prompt submission, tool execution, terminal response, and stop. Active injection, visible-input automation, and device voice routing are not baseline completion requirements.

### Firmware path

The firmware path is:

```text
Python host tests and component validation
  -> sequential PlatformIO v1 and v3 builds
  -> chip revision, partition, and budget validation
  -> matching factory image
  -> physical device only after chip identification
```

The v1 and v3 environments must not build concurrently in one working directory. Images must never be selected by guessing from a product title. The `ESP32-P4-WIFI6-M` suffix means soldered headers, not v3 silicon.

## Phased Delivery

### Phase 1: Source and web baseline

1. Record installed Node.js, npm, Python, Git, Rust, Cargo, and PlatformIO versions.
2. Install project-local JavaScript dependencies with `npm ci`.
3. Run desktop JavaScript tests, Bridge tests, the web production build, and source-safety checks.
4. Start the web-only UI and verify that it renders without claiming native capabilities.
5. Run firmware Python tests, protocol tests, and built-in component validation.

### Phase 2: Reproducible firmware builds

1. Install PlatformIO Core 6.1.19 in an isolated Python environment.
2. Build `esp32_p4_evboard` and then `esp32_p4_evboard_v3`.
3. Verify v1 uses its v1 dependency set and revision range 1-199.
4. Verify v3 uses its v3 dependency set and revision range 300-399.
5. Verify OTA headroom, internal static-memory budget, render stack budget, partition layout, and output artifacts.
6. Keep machine-specific paths out of dependency lock files.

### Phase 3: Native Windows and Claude Code

1. Install Rust stable, Cargo, MSVC Build Tools, WebView2, and reviewed runtime dependencies.
2. Run Rust library tests after desktop resources are prepared.
3. Launch the Tauri development application on Windows.
4. Verify managed ownership and cleanup of loopback ports 23333 and 8181.
5. Install Claude Hooks without losing existing user Hooks.
6. Verify a real Claude Code session produces one ordered working lifecycle and one terminal done state.
7. Verify restarting the Bridge does not replay historical completed sessions.

### Phase 4: Minimal hardware prototype

1. Obtain seller evidence for the board's actual ESP32-P4 silicon revision.
2. Select only the matching v1 or v3 factory image.
3. Connect the screen to DSI, not CSI, while power is disconnected.
4. Use the Windows host for the initial complete factory flash.
5. Verify device `hello`, firmware version, protocol schema, chip family, display output, status display, USB reconnect, and recovery from an interrupted connection.

Speech, joystick, touch, custom PCB, enclosure, and full audio validation belong to later phases.

## Minimal Procurement Plan

### First purchase

- One Waveshare `ESP32-P4-WIFI6` or `ESP32-P4-WIFI6-M` development board
  - 32 MB QSPI Flash
  - 32 MB in-package PSRAM
  - MIPI-DSI connector compatible with the target panel
  - Seller-confirmed silicon revision v1 or v3
- One `WLK2802MIPI-15P` compatible display module
  - 2.8 inch
  - Native 480 x 640 portrait resolution
  - ST7701S controller
  - One-lane MIPI-DSI
  - Matching 15-pin FPC pinout and initialization behavior
- One matching FPC cable
- One reliable USB-C data cable
- Any board-specific USB-path jumper required by the selected development board

The official reproduction page lists indicative prices of CNY 81.15 or more for the development board and CNY 65 for the display. The expected core subtotal is therefore about CNY 146.15 before cable, shipping, and price changes.

Before ordering, ask the board seller for the actual `chip_revision`, Flash and PSRAM capacities, DSI connector compatibility, CH343 availability, native USB HS support, and the USB-path switching method. Ask the display seller for the exact module number, full FPC pinout, lane count, panel voltages, reset and backlight wiring, and written compatibility with the target board. Do not accept “same controller” or “same pin count” as compatibility proof.

### Deferred purchase

After display and USB validation, add:

- Two 2.54 mm 1 x 20 female headers when needed
- One `YA13S-L7.4-B10Ka(60)-0-DL01` joystick
- Three Kailh BOX mechanical switches
- One 2011 miniature speaker with 1.25 mm connector
- Custom base PCB and assembly
- Enclosure
- M2.5 x 6 screws and M2.5 x 4 x 3.5 heat-set inserts
- Audio and microphone parts specified by the final board revision

## Error and Safety Rules

- A failure on either Bridge port makes the complete Bridge unhealthy.
- Invalid Claude settings fail closed and are not overwritten.
- Permission observation remains separate from permission approval.
- v1 and v3 images are never interchanged, forced, or selected by inference.
- Unknown chip revisions stop the flashing workflow.
- Initial installation uses a complete factory image; an application-only image is not a factory substitute.
- Factory flashing is destructive and occurs only after explicit device and image verification.
- No hardware claim is made from host tests alone.
- Every failure is first classified as source, dependency, platform, protocol, or hardware before code is changed.

## Verification and Completion Criteria

The reproduction baseline is complete when:

1. On WSL with Node.js 22 or newer:
   - `npm test` passes.
   - `npm run test:bridge` passes.
   - `npm run build:web` passes.
   - The web preview renders.
2. Firmware host checks pass:
   - `python -m pytest firmware/tests -q`
   - `python firmware/tests/protocol_contract_test.py`
   - `python firmware/tools/build_builtin_bundle.py --check`
3. PlatformIO 6.1.19 builds both v1 and v3 sequentially and produces the required bootloader, partition, OTA metadata, and application images with valid revision bounds and budgets.
4. Windows Tauri starts with its reviewed runtime resources and shuts down without orphaning managed Bridge processes.
5. A real Claude Code session produces correct ordered lifecycle state without duplicate terminal completion or historical replay.
6. After the minimal hardware arrives, the identified chip accepts only its matching factory image and passes display, handshake, basic state, and reconnect checks.

Until criterion 6 is complete, the project is described as source-reproduced and firmware-built, not hardware-validated.
