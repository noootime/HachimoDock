# 最小硬件基线实施计划

> **给执行此计划的代理：** 必须使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans，按任务逐步实施。步骤使用复选框（`- [ ]`）跟踪。

**目标：** 在买到的开发板已经确认芯片版本之后，只在 Windows 上烧录与之匹配的出厂镜像，并确认显示、握手、基本状态和重新连接。

**架构：** WSL 已经生成 v1 和 v3 出厂镜像。烧录在 Windows 上进行。`firmware/tools/flash_checked.py` 读取已连接芯片的版本，镜像范围不匹配时拒绝擦除。`firmware/tools/p4.ps1 factory-flash` 始终以 `esp32_p4_evboard` 为目标，所以它不能用来给一块尚未确认型号的板子选择镜像。

**技术栈：** esptool 5.4.0、`firmware/.pio/build/` 下的出厂镜像、Windows 串口。

## 全局约束

- 本计划不购买零件。下单前由用户提供卖家证据，烧录前由用户提供实物板卡。
- v1 与 v3 镜像不互换、不强制、也不靠推断选择。`ESP32-P4-WIFI6-M` 后缀表示焊有排针，不表示 v3 芯片。
- 芯片版本未知时，烧录流程停止。
- 首次安装使用完整出厂镜像 `pet-manager-p4-factory.bin`。只有应用分区的 `firmware.bin` 不能代替出厂镜像。
- 出厂烧录会清空设备。只有用户明确确认串口和所选镜像之后才执行。
- 屏幕接 DSI，不接 CSI，并且只在断电时插拔。
- 主机测试本身不构成硬件结论。
- 任何失败先归类为源码、依赖、平台、协议或硬件，然后再考虑修改产品代码。
- 语音、摇杆、触摸、定制 PCB、外壳和完整音频验证属于后续工作。

---

### 任务 1：下单前记录卖家证据

**文件：**
- 已有模板：`docs/superpowers/baselines/2026-10-29-hardware.md`

**接口：**
- 消费：开发板卖家和屏幕卖家的书面答复
- 产出：一份记录。它要么写明芯片是 v1 或 v3，要么写明还不能下单

- [ ] **步骤 1：填写已有记录**

模板已经在 `docs/superpowers/baselines/2026-10-29-hardware.md`。本步骤只填写卖家证据，烧录一节保持空白。

- [ ] **步骤 2：证据具体之前停下来**

用卖家的书面答复填写卖家证据。开发板只有在 `chip_revision` 为 v1 或 v3、Flash 为 32 MB、PSRAM 为 32 MB 时才接受。屏幕只有在具备精确模组编号、完整 15 针 FPC 引脚定义、一条 MIPI-DSI lane，以及针对该开发板的书面兼容说明时才接受。“同一个控制器”或“同样的针数”不够。任一必要答复缺失时，烧录一节留空并停止。代理会话里不下单。

### 任务 2：只读取已连接芯片，不写入

**文件：**
- 修改硬件记录中的 `识别到的版本` 一行

**接口：**
- 消费：Windows 上的 esptool 5.4.0，以及开发板的 USB-C 数据口
- 产出：一个数字形式的芯片版本，或者一次停止

- [ ] **步骤 1：安装固定版本的工具并读取芯片**

在 Windows 的干净虚拟环境中执行：

```powershell
python -m pip install esptool==5.4.0
python -m esptool version
python -m esptool --chip esp32p4 --port COMx chip-id
```

把 `COMx` 换成设备管理器里显示的端口。预期：esptool 报告 `5.4.0`，芯片名是 ESP32-P4，并且命令打印出芯片版本。这条命令不是写入或擦除命令。

- [ ] **步骤 2：对应版本范围**

记录整数版本。1 到 199 是 v1。300 到 399 是 v3。其他数值，或者读取失败，都停止流程。不根据产品标题选择镜像。

### 任务 3：选择匹配的出厂镜像

**文件：**
- 修改硬件记录中的 `选中的镜像` 一行

**接口：**
- 消费：任务 2 的版本，以及 WSL 上已经构建的镜像
- 产出：一个镜像路径，其头部记录的版本范围包含该芯片版本

- [ ] **步骤 1：检查与版本匹配的镜像**

版本 1–199 只使用 v1 镜像：

```text
firmware/.pio/build/esp32_p4_evboard/pet-manager-p4-factory.bin
```

版本 300–399 只使用 v3 镜像：

```text
firmware/.pio/build/esp32_p4_evboard_v3/pet-manager-p4-factory.bin
```

在仓库根目录，用能够导入 `firmware/tools/p4_image_compat.py` 的同一个 Python 运行：

```python
import sys
from pathlib import Path
sys.path.insert(0, "firmware/tools")
from p4_image_compat import factory_revision_range
image = Path(sys.argv[1])
print(image, factory_revision_range(image.read_bytes()))
```

v1 的预期是 `(1, 199)`。v3 的预期是 `(300, 399)`。已连接芯片的版本必须落在打印出的范围内。只把这一份文件复制到 Windows 上一个短的纯 ASCII 目录，例如 `C:\work\hachimodock-flash\pet-manager-p4-factory.bin`。另一份镜像不放进这个目录。

### 任务 4：断电连接屏幕

**文件：**
- `显示` 一行到任务 6 之后再改，本任务不改

- [ ] **步骤 1：开发板未通电时连接 DSI**

断开 USB 供电。把屏幕 FPC 接到开发板的 DSI 接口，不接 CSI。按任务 1 记录的引脚定义连接。接口插好之后再接回 USB。引脚定义没有写下来时，停下来，不猜测接法。

### 任务 5：明确确认之后才烧录

**文件：**
- 修改硬件记录中的 `端口` 一行

**接口：**
- 消费：`firmware/tools/flash_checked.py`、esptool 5.4.0、任务 3 选出的那一份镜像，以及用户的明确确认
- 产出：一块已烧录的开发板；检查失败时则不擦除

- [ ] **步骤 1：请求确认**

向用户展示串口、镜像路径和版本范围。只有用户明确表示对这个端口烧录这份镜像之后才继续。出厂烧录会清空设备。

- [ ] **步骤 2：运行带检查的烧录**

在 Windows 上，从仓库根目录执行：

```powershell
python firmware\tools\flash_checked.py --image C:\work\hachimodock-flash\pet-manager-p4-factory.bin --port COMx
```

预期：esptool 是 5.4.0，芯片是 ESP32-P4，已连接的版本落在镜像范围内。工具随后擦除、写入并复位。版本不匹配时抛出错误，并且不擦除设备。不加 `--force`。首次安装不调用 `p4.ps1 factory-flash`，因为该脚本始终构建并上传 `esp32_p4_evboard`。

### 任务 6：验证 hello、显示和重连

**文件：**
- 修改硬件记录中的 `hello`、`显示` 和 `重连` 三行

**接口：**
- 消费：`firmware/tools/p4.ps1` 中 `Read-P4HelloVersion` 的行为。它最多等待 18 秒，寻找 `"fw":"<version>"`
- 产出：基线里的硬件部分。完成之后，这个最小检查才可以称为硬件已验证

- [ ] **步骤 1：读取 hello**

按固件的监视波特率打开串口，一直读到 hello 帧出现。预期固件版本是 `0.7.103-p4-r1`，来自 `firmware/CMakeLists.txt` 的 `PROJECT_VER`。这一帧还要标明与任务 2 一致的芯片系列，以及这套源码使用的协议 schema。没有 hello 时，归类为硬件或协议失败。不为了让它出现而再次烧录。

- [ ] **步骤 2：检查屏幕和 USB 重连**

预期：屏幕显示状态界面，而不是黑屏。拔掉 USB，再插回同一个端口，再次读取 hello。预期：不再烧录也能读到同一个固件版本。这三件事都发生之后，三行结果才记为 `通过`。

- [ ] **步骤 3：只有烧录各行都通过才提交记录**

```powershell
git add docs/superpowers/plans/2026-10-29-reproduction-baseline-hardware.md docs/superpowers/baselines/2026-10-29-hardware.md
git commit -m "docs: record minimal hardware baseline"
```

卖家证据或烧录还不完整时，不提交一份写着通过的记录。
