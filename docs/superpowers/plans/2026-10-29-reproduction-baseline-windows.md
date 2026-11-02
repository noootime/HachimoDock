# Windows 与 Claude Code 基线实施计划

> **给执行此计划的代理：** 必须使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans，按任务逐步实施。步骤使用复选框（`- [ ]`）跟踪。

**目标：** 确认 Windows 原生应用能够启动并独占其回环 Bridge，Claude Code Hook 会合并进现有设置，并且一次真实 Claude Code 会话能留下有序的生命周期记录，重启后不会重放已完成的会话。

**架构：** 本计划的每条命令都在 Windows 主机上执行。WSL 的结果不能作为本计划的证据。React 只通过 Tauri 使用系统能力。受管 Bridge 监听 `127.0.0.1:23333`，Agent Session Bus 监听 `127.0.0.1:8181`。Hook 安装会合并进现有的 Claude 设置文件。权限观察与权限批准分开。

**技术栈：** Rust stable、Cargo、MSVC C++ Build Tools、WebView2、Node.js 22、已核对的可重定位 Node 与 FFmpeg 8.1.2 运行时、Tauri 2、Claude Code。

## 全局约束

- 在 Windows 上执行。WSL 中的运行结果不计入本计划。
- Windows 上的仓库路径使用短的纯 ASCII 路径，例如 `C:\work\HachimoDock`。Tauri 构建使用这份 Windows 工作副本，而不是经过路径转换的 WSL 路径。
- 分支是 `work/reproduction-baseline`。`origin` 保持为 `https://github.com/YizhengWw/HachimoDock.git`。
- 不提交 API Key、证书、私有运行时二进制、本机代理设置或生成的构建产物。
- 两个回环端口都正常时，Bridge 才算就绪。任一端口失败，整个 Bridge 都不健康。
- Hook 安装与现有 Claude 设置合并。无效的 Claude 设置按失败关闭处理，并且不被覆盖。
- 未知的权限策略保持为不批准。
- 本计划验证的生命周期是只读的：会话开始、提交提示、工具执行、终端响应、停止。主动注入、可见输入框自动化和设备语音路由不在范围内。
- 重启 Bridge 后，已完成的历史会话不会被重放。
- 任何失败先归类为源码、依赖、平台、协议或硬件，然后再考虑修改产品代码。
- 本计划不烧录硬件。

---

### 任务 1：记录 Windows 工具链

**文件：**
- 已有模板：`docs/superpowers/baselines/2026-10-29-windows.md`
- 测试：读这份记录

**接口：**
- 消费：Windows 主机上的工具
- 产出：一份基线记录。到任务 6 再提交。

- [ ] **步骤 1：在 Windows 上准备仓库**

Windows 宿主机默认没有这个仓库。把 WSL 里的仓库复制到 Windows 的 NTFS 目录。复制时排除 Linux 专用产物：`node_modules`、`.pio`、`__pycache__`、`.pytest_cache`、`managed_components`、`.superpowers`，以及 Linux 构建生成的 `firmware\sdkconfig.esp32_p4_evboard` 和 `firmware\sdkconfig.esp32_p4_evboard_v3`。`firmware\sdkconfig.defaults`、`firmware\sdkconfig.v3.defaults` 和测试夹具里的 `fake-openclaw\dist` 是仓库内容，要保留。Windows 上的 Tauri 构建使用 `C:\` 的本地副本，不使用 `\\wsl.localhost` 路径。

在 PowerShell 中执行。发行版名称为 `Ubuntu`：

```powershell
New-Item -ItemType Directory -Force C:\Users\shiyu.niu\sourcecode | Out-Null
robocopy "\\wsl.localhost\Ubuntu\home\shiyu\sourcecode\HachimoDock" "C:\Users\shiyu.niu\sourcecode\HachimoDock" /E /XD node_modules .pio __pycache__ .pytest_cache managed_components .superpowers /XF sdkconfig.esp32_p4_evboard sdkconfig.esp32_p4_evboard_v3
```

`robocopy` 的退出码 0 到 7 都表示成功，8 及以上表示有失败。之后进入副本目录核对：

```powershell
cd C:\Users\shiyu.niu\sourcecode\HachimoDock
[System.Environment]::OSVersion.VersionString
Get-Location
git branch --show-current
git remote get-url origin
git status --short
```

预期：路径为 `C:\Users\shiyu.niu\sourcecode\HachimoDock`，分支是 `work/reproduction-baseline`，origin 是 `https://github.com/YizhengWw/HachimoDock.git`。`git status` 应只列出未提交的计划与模板文件，即 `docs/superpowers/plans/` 和 `docs/superpowers/baselines/` 下的文件。如果出现大量 `modified` 而内容并没有改，多半是 NTFS 文件权限位的差异，把输出发给我，不要直接 `git add`。

基线记录文件（`docs/superpowers/baselines/2026-10-29-windows.md`）由 WSL 仓库提交。Windows 副本里的结果由你手动记录，然后把每个任务的输出发给我，我在 WSL 仓库里填写记录并提交。

- [ ] **步骤 2：记录工具**

模板已经在 `docs/superpowers/baselines/2026-10-29-windows.md`。用下面的命令填写工具链各行，保留结果各行为空：

```powershell
node -v
npm -v
rustc --version
cargo --version
cl
(Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}").pv
claude --version
```

预期：Node 22 或更新版本、Rust stable、来自 MSVC x64 开发环境的 `cl`、一个 WebView2 版本，以及一个 Claude Code 版本。缺哪个工具，就把它记为平台缺口，安装后再记录。不为了绕过缺口去改产品代码。

### 任务 2：准备已核对的运行时并运行 Rust 库测试

**文件：**
- 修改：`docs/superpowers/baselines/2026-10-29-windows.md` 中的 `prepare:desktop` 和 `cargo test --lib` 两行
- 生成：`pc/src-tauri/generated-runtime/`（已被忽略）

**接口：**
- 消费：官方可重定位的 Windows x64 Node、FFmpeg 8.1.2、它的 README、许可证、源码包和 `zlib.LICENSE`，说明见 `pc/docs/runtime-dependencies.md`
- 产出：准备好的桌面资源目录，以及通过的 `cargo test --lib`

- [ ] **步骤 1：把环境变量指到已核对的文件**

Windows x64 的 FFmpeg 二进制哈希必须是 `1cefca14ea713d5d7e7a1b948299142831185487f5efb14914d049ba9090d95f`。在 PowerShell 里只设置这些进程变量，并指向已核对的文件：

```powershell
$env:PET_MANAGER_NODE_BIN = "<official node.exe>"
$env:PET_MANAGER_FFMPEG_BIN = "<LGPL ffmpeg.exe>"
$env:PET_MANAGER_FFMPEG_README = "<ffmpeg.README>"
$env:PET_MANAGER_FFMPEG_LICENSE = "<ffmpeg.LICENSE>"
$env:PET_MANAGER_FFMPEG_SOURCE_ARCHIVE = "<FFmpeg-8.1.2-source.tar.xz>"
$env:PET_MANAGER_ZLIB_LICENSE = "<zlib.LICENSE>"
$env:PET_MANAGER_BUILD_FFMPEG_BIN = "<构建用 ffmpeg.exe>"
```

`PET_MANAGER_FFMPEG_BIN` 指向要打进安装包的 LGPL `ffmpeg.exe`。`PET_MANAGER_BUILD_FFMPEG_BIN` 指向另一个构建用 FFmpeg。未设置后者时，脚本使用 `PATH` 上的 `ffmpeg`。这个构建用 FFmpeg 不提交。时长探测始终需要它；只有缓存失效需要转码时才额外要求 `libx264`。不要用这份 LGPL `ffmpeg.exe` 充当构建用 FFmpeg，也不要用它做 `libx264` 转码。说明见 `pc/docs/runtime-dependencies.md`。这些路径和二进制文件不提交。

在 Windows 上，构建用 FFmpeg 用 winget 安装 Gyan.FFmpeg。9.0.2 含 `libx264`。安装后把 `PET_MANAGER_BUILD_FFMPEG_BIN` 指到该包里的 `ffmpeg.exe`，例如 `C:\Users\shiyu.niu\AppData\Local\Microsoft\WinGet\Packages\Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe\ffmpeg-9.0.2-full_build\bin\ffmpeg.exe`。winget 包目录会随包标识变化，以本机实际路径为准。

Rust 需要 1.95 或更新，因为锁文件里的 sysinfo 0.39.6 声明了这个要求。Windows 主机当前是 1.99.1。需要升级时执行 `rustup update stable`。不在仓库里添加 `rust-toolchain` 文件，也不要为了通过编译去改 `Cargo.toml`、`Cargo.lock` 或 sysinfo 版本。

- [ ] **步骤 2：准备并测试**

在 `pc/` 下、x64 Native Tools 开发者提示符里依次执行：

```powershell
npm run prepare:desktop
npm run build:web
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

在 `cargo test --lib` 之前要先在 `pc/` 执行 `npm run build:web`。`tauri.conf.json` 的 `frontendDist` 是 `../dist`，其中的 `terrier-clips` 会被构建脚本当作资源。`dist/` 被 git 忽略，干净的 Windows 副本里没有它。

预期：`prepare:desktop` 检查架构、版本和 FFmpeg 哈希，然后把文件复制到 `pc/src-tauri/generated-runtime/`。它退出码为 0 时记为 `通过`。哈希检查失败时停下来，归类为依赖问题。构建用 FFmpeg 缺失或无法完成探测/转码时，同样归类为依赖问题并停止。不修改预期哈希。`configure_storage_dir` 只在 macOS 存在，测试里对它的调用也只在 macOS 编译。`cargo test --lib` 在这次条件编译之后能够编译，但测试可执行文件启动时退出码是 `0xC0000139`，不要记为 `通过`。这个运行失败由 Windows 端继续查，不要为它改产品代码。

### 任务 3：启动 Tauri 并确认受管端口会退出

**文件：**
- 修改基线记录中的 `Tauri 启动与退出` 和 `端口 23333 和 8181` 两行

**接口：**
- 消费：任务 2 准备好的运行时，以及一份不含密钥的 `%USERPROFILE%\.claw-pet\pet-bridge.json`
- 产出：开发版应用能够启动，23333 和 8181 在运行期间监听，退出后不再被监听

**前置条件：**

本任务原先漏写了这一条。`pc/src-tauri/src/lib.rs` 在窗口打开约 3 秒后调用 `ensure_bridge_runtime_blocking`。`pet-bridge.json` 不存在，或其中 `desktopDeviceId` 为空，并且同目录的 `device-bindings.json` 没有可恢复的设备绑定时，函数返回 `inactive`，故意不监听 23333 和 8181。这不是端口启动失败。重跑下面的步骤之前，要先有一份不含密钥的 `pet-bridge.json`，其中 `desktopDeviceId` 为非空字符串。不要把 `mqttPassword` 或其它密钥写进这个文件。不要改 `lib.rs` 来强迫监听。没有这份文件时，不要把 Bridge 记为就绪，也不要把本任务的两行结果记为 `通过`。

- [ ] **步骤 1：启动开发版应用**

在 `pc/` 的 PowerShell 中执行：

```powershell
$env:PET_MANAGER_ALLOW_DEV_ORIGIN = "1"
npm run dev
```

预期：原生窗口打开。`npm run dev` 是 `prepare:desktop` 加上 `tauri dev`。这才是原生应用。`npm run dev:web` 不能满足本任务。

- [ ] **步骤 2：应用运行时检查两个端口**

```powershell
Get-NetTCPConnection -LocalAddress 127.0.0.1 -LocalPort 23333,8181 -State Listen
```

预期：前置条件满足后，23333 和 8181 各有一个监听。缺任何一个，Bridge 就不健康。停下来并归类失败原因。没有不含密钥的 `pet-bridge.json` 时，两个端口不监听是源码的 `inactive` 行为，归类为源码，不把 Bridge 记为就绪。

- [ ] **步骤 3：退出并确认受管监听已经消失**

用应用自己的退出路径关闭它。然后执行：

```powershell
Get-NetTCPConnection -LocalAddress 127.0.0.1 -LocalPort 23333,8181 -State Listen -ErrorAction SilentlyContinue
```

预期：没有监听。只有前置条件已经满足，并且运行期间两个端口都曾监听、退出后又都消失，两行结果才记为 `通过`。残留进程属于受管清理失败，归类为平台问题。

### 任务 4：合并 Claude Hook，保留现有设置

**文件：**
- 修改基线记录中的 `Claude Hook 合并` 一行
- 仓库之外：用户的 Claude 设置文件，只通过安装器修改

**接口：**
- 消费：`pc/src-tauri/bridge/hooks/install.js`。它追加 Hook 条目；设置文件存在但无法解析时会抛出错误
- 产出：设置文件仍包含用户原来的 Hook，并加上 Clawd Hook

- [ ] **步骤 1：给当前设置做快照**

```powershell
$settings = Join-Path $env:USERPROFILE ".claude\settings.json"
Copy-Item -LiteralPath $settings -Destination "$env:TEMP\claude-settings.before.json" -ErrorAction SilentlyContinue
Get-FileHash -Algorithm SHA256 $settings -ErrorAction SilentlyContinue
```

文件不存在时，记录安装器可以创建它。文件存在时，保留这份副本，直到本任务通过。

- [ ] **步骤 2：证明无效设置不会被覆盖**

如果设置文件存在，先把它另外复制一份，再把正在使用的文件替换成单个字符 `{`，然后运行：

```powershell
node pc\src-tauri\bridge\hooks\install.js
```

预期：非零退出，并输出 `Failed to read settings.json`。文件内容仍然正好是 `{`。继续之前恢复快照：

```powershell
Copy-Item -LiteralPath "$env:TEMP\claude-settings.before.json" -Destination $settings -Force
```

- [ ] **步骤 3：安装 Hook 并比较**

```powershell
node pc\src-tauri\bridge\hooks\install.js
```

预期：退出码为 0。把恢复后再更新的文件与快照比较。原先的 Hook 命令仍然存在。新增的 Hook 命令指向 `clawd-hook.js`。权限决定里没有无条件允许。把 `Claude Hook 合并` 记为 `通过`。

### 任务 5：观察一次 Claude Code 生命周期和一次重启

**文件：**
- 修改基线记录中的 `Claude 生命周期` 和 `Bridge 重启重放` 两行

**接口：**
- 消费：正在运行的 Tauri 应用，以及任务 3 和任务 4 合并好的 Hook
- 产出：一次有序的工作中生命周期和一个终态 done，然后一次重启，这次已完成的会话不会被重放

- [ ] **步骤 1：跑一次真实的 Claude Code 会话**

再次启动 Tauri 应用。在另一个 Windows 终端里，于一个空目录启动 Claude Code，提交一条会使用一个工具的短提示，等这一轮结束并停止会话。

预期应用的会话界面按这个顺序出现：会话开始、提交提示、工具执行、终端响应、停止。会话到达一个终态 done。同一轮不会再产生第二个 done 事件。把观察到的顺序写进基线记录。不用自动化向 Claude 界面输入。

- [ ] **步骤 2：重启 Bridge**

退出 Tauri 应用，再启动一次。不要开始新的 Claude 会话。

预期：步骤 1 里已完成的会话不会被重放成新的工作中生命周期，也不会出现新的 done 事件。只有在这一点成立时，才把 `Bridge 重启重放` 记为 `通过`。历史文本可以继续作为历史显示。已经结束的会话如果出现新的生命周期转换，就是失败。

### 任务 6：记录 Windows 基线

**文件：**
- 修改：`docs/superpowers/baselines/2026-10-29-windows.md`

- [ ] **步骤 1：收尾记录**

每一行结果都写 `通过`。然后加上：

```markdown
## 状态

Windows Tauri 启动、受管端口清理、Claude Hook 合并，以及一次只读的 Claude Code 生命周期已经通过。这不构成硬件验证。
```

- [ ] **步骤 2：在 WSL 仓库里提交基线文档**

这一步不在 Windows 克隆里做。基线记录和本计划文件只存在于 WSL 仓库，由 WSL 端提交：

```bash
cd ~/sourcecode/HachimoDock
git add docs/superpowers/plans/2026-10-29-reproduction-baseline-windows.md docs/superpowers/baselines/2026-10-29-windows.md
git diff --cached --stat
git commit -m "docs: record Windows reproduction baseline"
```

预期：暂存区里只有这两份文档。生成的运行时文件、Claude 设置和固件镜像不暂存。前面任一任务失败时，不提交一份写着通过的记录。
