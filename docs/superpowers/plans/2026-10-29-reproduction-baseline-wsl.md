# WSL Reproduction Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the current `main` source builds and tests on WSL2, including sequential ESP32-P4 v1 and v3 firmware images, without changing product behavior.

**Architecture:** WSL2 owns Node.js, the web UI, Python host tests, and PlatformIO. JavaScript dependencies stay in `pc/node_modules`. PlatformIO 6.1.19 lives in a virtual environment outside the repository. v1 and v3 are built one after another in this working tree. Generated images stay untracked. Windows Tauri, Claude Code, and physical hardware are outside this plan.

**Tech Stack:** Node.js 22, npm, Vite, Python 3.12, pytest, Pillow, PlatformIO Core 6.1.19, the repository's existing Node test runner and firmware host tests.

## Global Constraints

- Working directory is `~/sourcecode/HachimoDock` on WSL ext4, not `/mnt/c`.
- Branch is `work/reproduction-baseline`. `origin` stays `https://github.com/YizhengWw/HachimoDock.git`.
- Do not commit API keys, certificates, private runtime binaries, machine-specific proxy settings, generated build output, `node_modules/`, `firmware/.pio/`, or `firmware/sdkconfig.*` other than the already tracked defaults.
- `npm run dev:web` is a UI-only preview and does not provide Tauri IPC, USB, system permissions, local Agent detection, or managed Bridge behavior.
- PlatformIO Core is 6.1.19. v1 environment is `esp32_p4_evboard` with revision range 1–199. v3 environment is `esp32_p4_evboard_v3` with revision range 300–399.
- Do not build v1 and v3 concurrently in this working directory.
- The `ESP32-P4-WIFI6-M` suffix means soldered headers, not v3 silicon. Do not select an image from a product title.
- A failure is classified as source, dependency, platform, protocol, or hardware before any product code is changed. This plan does not authorize a product-code change.
- No hardware claim is made from these host results.
- Phase 3 (native Windows and Claude Code) and Phase 4 (minimal hardware) from `docs/superpowers/specs/2026-10-29-reproduction-baseline-design.md` are separate future plans.

---

### Task 1: Record the WSL toolchain

**Files:**
- Create: `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Modify: none
- Test: the new note, by reading it

**Interfaces:**
- Consumes: the installed system tools
- Produces: a baseline note whose Toolchain section later tasks append to. Do not commit it until Task 8.

- [ ] **Step 1: Confirm the clean product tree**

Run from `~/sourcecode/HachimoDock`:

```bash
git status --short --branch
git rev-parse --short HEAD
git remote get-url origin
```

Expected:

- Branch `work/reproduction-baseline`
- HEAD `2e8de4e` unless a later commit on this branch already exists
- Origin `https://github.com/YizhengWw/HachimoDock.git`
- The only allowed untracked path before this task is `docs/superpowers/plans/2026-10-29-reproduction-baseline-wsl.md` and an untracked `.vscode/` directory. No product source is modified.

- [ ] **Step 2: Write the toolchain section**

Create `docs/superpowers/baselines/2026-10-29-wsl-source.md` with this exact body, then replace each command's output line with the live output:

```markdown
# WSL source and firmware baseline — 2026-10-29

Environment: WSL2. These results do not validate Windows Tauri, USB, audio, installers, or physical hardware.

## Toolchain

Recorded before installing project dependencies.

- node: `node -v`
- npm: `npm -v`
- python3: `python3 --version`
- git: `git --version`
- cc: `cc --version | head -n 1`
- rustc: absent on WSL; Windows owns Rust
- cargo: absent on WSL; Windows owns Cargo
- platformio: absent until Task 4

## Results

- source safety:
- npm test:
- npm run test:bridge:
- npm run build:web:
- web preview:
- pytest:
- protocol contract:
- builtin bundle:
- PlatformIO:
- v1 image:
- v3 image:
- dependency locks:
```

Known values at plan time, which the live commands must confirm rather than copy blindly: Node `v22.21.0`, npm `10.9.4`, Python `3.12.3`, Git `2.43.0`, `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`. Rust, Cargo, and PlatformIO were absent.

- [ ] **Step 3: Check the note**

```bash
test -f docs/superpowers/baselines/2026-10-29-wsl-source.md
grep -n "WSL2" docs/superpowers/baselines/2026-10-29-wsl-source.md
```

Expected: the file exists and names WSL2. Do not commit yet.

### Task 2: Install JavaScript dependencies and run the source checks

**Files:**
- Modify: `docs/superpowers/baselines/2026-10-29-wsl-source.md` result lines for source safety, `npm test`, `npm run test:bridge`, and `npm run build:web`
- Test: existing `pc` scripts. No new test file.

**Interfaces:**
- Consumes: Node.js 22 from Task 1
- Produces: `pc/node_modules/` and a passing production web build under the ignored `pc/dist/` directory

- [ ] **Step 1: Remove the local editor path that fails the publication check**

`pc/scripts/check-source-safety.mjs` rejects any scanned file that contains the current user's home directory followed by a slash. The untracked file `.vscode/settings.json` currently sets `cmake.sourceDirectory` to this repository's `firmware/` directory with that absolute personal path.

```bash
mkdir -p /tmp/hachimodock-local
mv .vscode/settings.json /tmp/hachimodock-local/settings.json
rmdir .vscode
```

Expected: `.vscode/` is gone from the repository. Do not commit that settings file and do not move it back. It is editor state, not product source.

- [ ] **Step 2: Run the source publication check**

From the repository root:

```bash
node pc/scripts/check-source-safety.mjs
```

Expected: exit 0 and one line beginning `Source publication checks passed for `.

- [ ] **Step 3: Install locked JavaScript dependencies**

```bash
npm ci --prefix pc
```

Expected: exit 0. `pc/node_modules/` exists. `pc/package-lock.json` has no diff:

```bash
git diff --exit-code -- pc/package-lock.json
```

- [ ] **Step 4: Run desktop tests, Bridge tests, and the production web build**

```bash
npm test --prefix pc
npm run test:bridge --prefix pc
npm run build:web --prefix pc
```

Expected: each command exits 0. `npm test` includes the provenance check because `pc/package.json` defines `test` as `npm run check:provenance && node scripts/run-node-tests.mjs src`. `build:web` runs `check:public-build-env`, `vite build`, and `check:release-csp`.

- [ ] **Step 5: Record the four results and confirm no product diff**

Set the matching result lines in `docs/superpowers/baselines/2026-10-29-wsl-source.md` to `pass` plus the exit code. Then:

```bash
git status --short
```

Expected tracked changes: only `docs/superpowers/baselines/2026-10-29-wsl-source.md` and, if still untracked, `docs/superpowers/plans/2026-10-29-reproduction-baseline-wsl.md`. `pc/node_modules/` and `pc/dist/` must not appear.

If a command fails, stop. Classify it before editing product code.

### Task 3: Verify the web-only preview

**Files:**
- Modify: the `web preview` line in `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: the running Vite dev server. No new test file.

**Interfaces:**
- Consumes: `pc/node_modules/` from Task 2
- Produces: evidence that `http://127.0.0.1:4173` renders the dev preview without a Tauri runtime

- [ ] **Step 1: Start the UI-only server**

From `pc/`:

```bash
npm run dev:web
```

`pc/package.json` defines this as `vite --host 0.0.0.0 --port 4173`. Leave it running. Expected log includes `Local:` and port `4173`.

- [ ] **Step 2: Check the HTML entry**

```bash
curl --silent --fail http://127.0.0.1:4173/ | tee /tmp/hachimodock-web-preview.html
```

Expected: exit 0, `<title>HachimiDock</title>`, `<div id="root"></div>`, and `<script type="module" src="/src/main.jsx"></script>`.

- [ ] **Step 3: Check the rendered page in a browser**

Open `http://127.0.0.1:4173/`. In the page runtime, evaluate:

```javascript
({
  tauri: window.__TAURI_INTERNALS__,
  rootChildren: document.querySelector("#root")?.childElementCount ?? 0,
  text: document.body.innerText,
})
```

Expected:

- `tauri` is `undefined`
- `rootChildren` is greater than 0
- `text` contains `board-dev-direct-001`, the dev-only device ID from `DEV_DIRECT_DASHBOARD_BINDING` in `pc/src/App.jsx`, and `USB 未连接`

The dev-only binding is used when `import.meta.env.DEV` is true and `window.__TAURI_INTERNALS__` is absent. Its `wifiSsid` value is not rendered unless a USB port is connected, so do not look for it in the page text. This is a UI preview check only, not a USB device check.

**Revision (2026-10-29):** the original expected text `开发直达` was wrong. It is not rendered on the page. The revised check was confirmed by a browser run that showed the page rendering normally with no Tauri runtime.

- [ ] **Step 4: Stop the server and record the result**

Stop the Vite process started in Step 1. Confirm nothing is still listening:

```bash
ss -ltnp | grep 4173 || true
```

Expected: no listener on 4173. Set `web preview` in the baseline note to `pass`. A stopped preview does not exercise ports 23333 or 8181; those belong to the later Windows plan.

### Task 4: Install PlatformIO 6.1.19 outside the repository

**Files:**
- Create: `$HOME/.local/share/hachimodock/pio-6.1.19/` (outside the Git repository)
- Modify: the `PlatformIO` line in `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: `python -m platformio --version`

**Interfaces:**
- Consumes: system Python 3.12 and `python3-venv`
- Produces: `PIO_PY=$HOME/.local/share/hachimodock/pio-6.1.19/bin/python`, used by Tasks 5–8

- [ ] **Step 1: Create the virtual environment and install the pinned tools**

```bash
python3 -m venv $HOME/.local/share/hachimodock/pio-6.1.19
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m pip install --upgrade pip
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m pip install 'platformio==6.1.19' pytest Pillow
```

Expected: both pip commands exit 0. This environment is intentionally outside the repository so it cannot be committed.

- [ ] **Step 2: Verify the Core version**

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m platformio --version
```

Expected: a version line containing `6.1.19`. Set the baseline `PlatformIO` line to that exact output. Core 6.1.18 cannot build the v3 platform.

### Task 5: Run firmware host checks

**Files:**
- Modify: the `pytest`, `protocol contract`, and `builtin bundle` lines in `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: `firmware/tests/`, `firmware/tests/protocol_contract_test.py`, and `firmware/tools/build_builtin_bundle.py --check`

**Interfaces:**
- Consumes: `PIO_PY` from Task 4 and system `cc`
- Produces: host-test evidence only. No firmware image yet.

- [ ] **Step 1: Run the pytest suite**

From the repository root:

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m pytest firmware/tests -q
```

Expected: exit 0. Several tests invoke `cc`; a missing compiler is a platform failure, not a reason to change firmware source.

- [ ] **Step 2: Run the protocol contract script**

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python firmware/tests/protocol_contract_test.py
```

Expected: exit 0 and lines of the form `ok <test_name>`.

- [ ] **Step 3: Check the built-in component bundle**

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python firmware/tools/build_builtin_bundle.py --check
```

Expected: exit 0. Then confirm the check did not rewrite tracked inputs:

```bash
git diff --exit-code -- firmware/main/pet_p4_builtin_components.json firmware/main/pet_p4_builtin_components_data.c
```

Expected: empty diff. Record all three results as `pass`.

### Task 6: Build and verify the v1 firmware

**Files:**
- Modify: the `v1 image` line in `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: generated files under the ignored directory `firmware/.pio/build/esp32_p4_evboard/`
- Read-only inputs: `firmware/platformio.ini`, `firmware/partitions.csv`, `firmware/tools/p4_image_compat.py`, `firmware/tools/firmware_budget.py`

**Interfaces:**
- Consumes: `PIO_PY` from Task 4
- Produces: v1 bootloader, partition table, OTA metadata, application, budget report, and factory image. v3 must not be building while this task runs.

- [ ] **Step 1: Build v1, then its factory image**

The first run downloads the pinned pioarduino platform `55.03.32`, ESP-IDF 5.5.1, and the P4 toolchain. It needs network access and can take a long time.

```bash
cd firmware
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m platformio run -e esp32_p4_evboard
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m platformio run -e esp32_p4_evboard -t factory
```

Expected: both commands exit 0. Do not start `esp32_p4_evboard_v3` in another process. Do not pass `--force` or lower the revision bounds.

- [ ] **Step 2: Verify revision bounds, budgets, and artifacts**

From the repository root:

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python - <<'PY'
import json
import sys
from pathlib import Path

sys.path.insert(0, "firmware/tools")
from p4_image_compat import factory_revision_range, image_revision_range

env = "esp32_p4_evboard"
expected = (1, 199)
build = Path("firmware/.pio/build") / env
names = [
    "bootloader.bin",
    "partitions.bin",
    "ota_data_initial.bin",
    "firmware.bin",
    "firmware.factory.bin",
    "firmware-budget.json",
    "pet-manager-p4-factory.bin",
    "pet-manager-p4-factory.json",
]
for name in names:
    path = build / name
    if not path.is_file() or path.stat().st_size == 0:
        raise SystemExit(f"missing or empty {path}")
for name in ("bootloader.bin", "firmware.bin"):
    got = image_revision_range((build / name).read_bytes())
    if got != expected:
        raise SystemExit(f"{name} bounds {got} != {expected}")
factory = factory_revision_range((build / "pet-manager-p4-factory.bin").read_bytes())
if factory != expected:
    raise SystemExit(f"factory bounds {factory} != {expected}")
sdkconfig = Path("firmware/sdkconfig.esp32_p4_evboard").read_text(encoding="utf-8")
for line in ("CONFIG_ESP_REV_MIN_FULL=1", "CONFIG_ESP_REV_MAX_FULL=199"):
    if line not in sdkconfig.splitlines():
        raise SystemExit(f"sdkconfig missing {line}")
budget = json.loads((build / "firmware-budget.json").read_text(encoding="utf-8"))
if budget["otaHeadroomBytes"] < 128 * 1024:
    raise SystemExit(f"OTA headroom below 128 KiB: {budget}")
if budget["internalStaticBytes"] > 384 * 1024:
    raise SystemExit(f"internal static memory exceeds 384 KiB: {budget}")
lock = Path("firmware/dependencies.lock").read_text(encoding="utf-8")
if "version: 5.5.1" not in lock:
    raise SystemExit("v1 dependency lock is not the 5.5.1 set")
print("ok", env, expected, budget)
PY
```

Expected: one line starting `ok esp32_p4_evboard (1, 199)`.

The successful `pio run` already enforces the 4096-byte render-stack budget in `firmware/scripts/platformio_chip_images.py`. A stack overrun fails that command in Step 1.

- [ ] **Step 3: Verify the partition layout used by the image**

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python - <<'PY'
import csv
from pathlib import Path

expected = {
    "nvs": ("0x9000", "0x6000"),
    "phy_init": ("0xf000", "0x1000"),
    "ota_0": ("0x10000", "0x280000"),
    "ota_1": ("0x290000", "0x280000"),
    "otadata": ("0x510000", "0x2000"),
    "storage": ("0x520000", "0x6e0000"),
    "appearance0": ("0xc00000", "0xa00000"),
    "appearance1": ("0x1600000", "0xa00000"),
}
rows = {}
for row in csv.reader(Path("firmware/partitions.csv").read_text(encoding="utf-8").splitlines()):
    if not row or row[0].strip().startswith("#"):
        continue
    name, offset, size = row[0].strip(), row[3].strip().lower(), row[4].strip().lower()
    rows[name] = (offset, size)
if rows != expected:
    raise SystemExit(f"partition layout changed: {rows}")
print("ok partitions")
PY
```

Expected: `ok partitions`. Set `v1 image` to `pass` with the printed revision tuple and budget JSON. Leave every generated image untracked.

### Task 7: Build and verify the v3 firmware

**Files:**
- Modify: the `v3 image` line in `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: generated files under `firmware/.pio/build/esp32_p4_evboard_v3/`

**Interfaces:**
- Consumes: Task 6 finished. v1 and v3 must not overlap.
- Produces: v3 images bounded to revision 300–399 and dependency lock `firmware/dependencies.v3.lock` at ESP-IDF 5.5.4

- [ ] **Step 1: Build v3 only after the v1 process has exited**

```bash
cd firmware
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m platformio run -e esp32_p4_evboard_v3
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python -m platformio run -e esp32_p4_evboard_v3 -t factory
```

Expected: both commands exit 0. This uses pioarduino platform `55.03.38-1` and ESP-IDF 5.5.4. Do not reuse the v1 factory image for this environment.

- [ ] **Step 2: Verify v3 bounds, budgets, and artifacts**

From the repository root, run the same Python checker as Task 6 Step 2 with these two assignments changed:

```python
env = "esp32_p4_evboard_v3"
expected = (300, 399)
```

and these two later checks changed:

```python
sdkconfig = Path("firmware/sdkconfig.esp32_p4_evboard_v3").read_text(encoding="utf-8")
for line in ("CONFIG_ESP_REV_MIN_FULL=300", "CONFIG_ESP_REV_MAX_FULL=399", "CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ_400=y"):
    if line not in sdkconfig.splitlines():
        raise SystemExit(f"sdkconfig missing {line}")
lock = Path("firmware/dependencies.v3.lock").read_text(encoding="utf-8")
if "version: 5.5.4" not in lock:
    raise SystemExit("v3 dependency lock is not the 5.5.4 set")
```

The artifact names stay the same. Expected output starts with `ok esp32_p4_evboard_v3 (300, 399)`. Set `v3 image` to `pass` with that tuple and budget JSON.

The full checker is:

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python - <<'PY'
import json
import sys
from pathlib import Path

sys.path.insert(0, "firmware/tools")
from p4_image_compat import factory_revision_range, image_revision_range

env = "esp32_p4_evboard_v3"
expected = (300, 399)
build = Path("firmware/.pio/build") / env
names = [
    "bootloader.bin",
    "partitions.bin",
    "ota_data_initial.bin",
    "firmware.bin",
    "firmware.factory.bin",
    "firmware-budget.json",
    "pet-manager-p4-factory.bin",
    "pet-manager-p4-factory.json",
]
for name in names:
    path = build / name
    if not path.is_file() or path.stat().st_size == 0:
        raise SystemExit(f"missing or empty {path}")
for name in ("bootloader.bin", "firmware.bin"):
    got = image_revision_range((build / name).read_bytes())
    if got != expected:
        raise SystemExit(f"{name} bounds {got} != {expected}")
factory = factory_revision_range((build / "pet-manager-p4-factory.bin").read_bytes())
if factory != expected:
    raise SystemExit(f"factory bounds {factory} != {expected}")
sdkconfig = Path("firmware/sdkconfig.esp32_p4_evboard_v3").read_text(encoding="utf-8")
for line in ("CONFIG_ESP_REV_MIN_FULL=300", "CONFIG_ESP_REV_MAX_FULL=399", "CONFIG_ESP_DEFAULT_CPU_FREQ_MHZ_400=y"):
    if line not in sdkconfig.splitlines():
        raise SystemExit(f"sdkconfig missing {line}")
budget = json.loads((build / "firmware-budget.json").read_text(encoding="utf-8"))
if budget["otaHeadroomBytes"] < 128 * 1024:
    raise SystemExit(f"OTA headroom below 128 KiB: {budget}")
if budget["internalStaticBytes"] > 384 * 1024:
    raise SystemExit(f"internal static memory exceeds 384 KiB: {budget}")
lock = Path("firmware/dependencies.v3.lock").read_text(encoding="utf-8")
if "version: 5.5.4" not in lock:
    raise SystemExit("v3 dependency lock is not the 5.5.4 set")
print("ok", env, expected, budget)
PY
```

### Task 8: Confirm portable dependency locks and record the baseline

**Files:**
- Modify: `docs/superpowers/baselines/2026-10-29-wsl-source.md`
- Test: `git diff` on the two dependency locks

**Interfaces:**
- Consumes: both firmware builds from Tasks 6 and 7
- Produces: a committed baseline note. The repository is then source-reproduced and firmware-built, not hardware-validated.

- [ ] **Step 1: Normalize and inspect both lock files**

```bash
$HOME/.local/share/hachimodock/pio-6.1.19/bin/python firmware/tools/normalize_dependency_lock.py
git diff --exit-code -- firmware/dependencies.lock firmware/dependencies.v3.lock
git grep -n -E '/home/|/Users/|[A-Za-z]:\\' -- firmware/dependencies.lock firmware/dependencies.v3.lock
```

Expected:

- The normalizer prints `Local dependency paths are portable`
- `git diff` exits 0
- `git grep` exits 1 because neither lock contains a machine path

If the diff is not empty after normalization, stop and classify it. Do not commit the lock files.

- [ ] **Step 2: Finish the baseline note**

Set `dependency locks` to `pass`. Every Results line must say `pass`. Add this closing paragraph:

```markdown
## Status

WSL source checks and both sequential PlatformIO 6.1.19 firmware builds passed. The project is source-reproduced and firmware-built. It is not hardware-validated. Windows Tauri, Claude Code lifecycle observation, and device flashing remain separate work.
```

- [ ] **Step 3: Commit only the baseline documents**

```bash
git add docs/superpowers/plans/2026-10-29-reproduction-baseline-wsl.md docs/superpowers/baselines/2026-10-29-wsl-source.md
git diff --cached --stat
git commit -m "$(cat <<'EOF'
docs: record WSL reproduction baseline

EOF
)"
git status --short --branch
```

Expected cached stat contains only those two documentation paths. `git status` after the commit shows a clean tracked tree. Generated firmware, `pc/node_modules/`, `pc/dist/`, and `firmware/sdkconfig.esp32_p4_evboard*` remain untracked or ignored.

If any earlier task failed, do not commit a passing baseline note.
