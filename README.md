# Qzz Mirror · 轻镜

一个用于 Windows 的简洁安卓投屏控制面板。基于官方 [scrcpy](https://github.com/Genymobile/scrcpy)，提供设备选择、USB / Wi-Fi 连接和无线配置。投屏画面使用独立的 scrcpy 窗口，电脑键鼠操作直接由 scrcpy 完成。

[下载最新版](https://github.com/Mai-Mu/qzz-mirror/releases/latest) · [更新日志](CHANGELOG.md)

首版面向个人使用，一次投屏一台设备。已确认范围见 [产品决策](docs/decisions.md)，验证项目见 [验收清单](docs/acceptance.md)。

## 解压使用

1. 将 `Qzz-Mirror-0.2.1-win-x64.zip` 完整解压到可读写目录。
2. 保留 `qzz-mirror.exe` 与旁边的 `runtime` 文件夹，双击 `qzz-mirror.exe`。
3. 连接设备，在面板中选择它，点击“开始投屏”。打开软件本身不会自动投屏。

运行环境为 Windows 10 / 11 x64，需要 Microsoft Edge WebView2 Runtime。开发工具不属于用户运行依赖；若本机缺少 WebView2，可按 [Tauri 官方 Windows 依赖说明](https://v2.tauri.app/start/prerequisites/#windows)准备。便携包已包含 scrcpy、adb 及其运行文件，无需另配 adb 环境变量。

### 第一次通过 USB 连接

1. 在设备上开启“开发者选项 → USB 调试”。
2. 使用支持数据传输的数据线连接电脑，并在设备上允许这台电脑调试。设备显示“未授权”时，需要先在设备屏幕上确认。
3. 在轻镜中刷新设备列表，选中设备并点击“开始投屏”。

“自动”连接方式会在**所选设备**内部优先使用 USB；没有可用 USB 时再使用该设备已连接的 Wi-Fi。USB 与 Wi-Fi 同时在线时，可手动选择连接方式，不需要自己输入设备序列号。

### 先用 USB 配置无线

1. 手机与电脑接入同一局域网，保持 USB 已连接并授权。
2. 打开“配置无线”，选择“USB 辅助配置”。
3. 等待无线连接验证成功。可以选择 Wi-Fi 后开始投屏，再拔掉数据线。

使用 USB 投屏时拔线，轻镜会对同一设备已配置的无线连接尝试恢复；恢复会重新打开 scrcpy 窗口，画面有短暂中断。失败时查看原因并点击重试。设备重启、网络变化或无线地址失效后，可能需要重新插 USB 配置，不能保证永久免插线。

### Android 11 及以上使用配对码

1. 在设备“开发者选项 → 无线调试”中开启无线调试，打开“使用配对码配对设备”。
2. 在轻镜“配置无线 → 配对码连接”中输入该页面显示的配对地址、端口和 6 位配对码。
3. 连接地址填写“无线调试”**主页**显示的 IP 地址和端口。如果自动发现没有找到设备，使用这个连接地址再次连接。

**配对端口与投屏连接端口通常不同。** 配对弹窗的端口用于授权配对，主页的端口用于后续连接；不要把弹窗端口当作保存的连接端口。端口或 IP 改变后应使用设备当前显示的值。参考 [Android 官方无线 adb 说明](https://developer.android.com/tools/adb#wireless-android11-command-line)。

### 已保存的无线设备

点击已保存设备，核对当前 IP 和端口后连接，再点击“开始投屏”。保存地址不会让处于关机、离网或已关闭调试状态的设备自动上线。

### 设置与托盘

- 主页保留设备、连接和投屏操作；点击齿轮进入应用内的设置页，返回后继续操作设备。
- 帧率默认为“最高”，不额外限帧；也可输入 1～240 的整数作为帧率上限。实际帧率取决于设备、画面更新和连接性能。
- 分辨率固定使用设备原始尺寸，没有分辨率或三档画质控件。声音转发与投屏置顶在设置页分别调整，保存后重新开始投屏生效。
- Android 9 不能转发声音，轻镜会关闭该设备的音频；画面和键鼠操作可以继续。scrcpy 音频转发要求 Android 11 及以上，Android 11 开始投屏时还需保持设备解锁。参考 [scrcpy 音频文档](https://github.com/Genymobile/scrcpy/blob/v5.0.1/doc/audio.md)。
- 关闭控制面板会进入系统托盘，投屏继续。通过托盘打开面板；需要彻底退出时，右键托盘图标选择“退出”。退出会结束轻镜自己启动的投屏会话。
- 在面板点击“停止投屏”，或关闭独立投屏窗口，可以结束当前投屏。

## 常见连接提示

| 提示或现象 | 处理方式 |
| --- | --- |
| 未找到设备 | 检查数据线、USB 调试和手机授权，然后刷新。无线设备还需确认局域网地址与端口。 |
| 设备未授权 | 解锁设备，在“允许 USB 调试”弹窗中确认。 |
| 设备离线 / 连接超时 | 检查设备是否在线、IP 是否改变、无线调试是否开启，以及网络是否允许两台设备互通。 |
| USB 和 Wi-Fi 同时存在 | 在轻镜选择设备及连接方式后开始，无需修改 scrcpy 启动脚本。 |
| 配对成功但没有设备 | 核对无线调试主页的连接端口，使用“手动连接”，不要填写配对弹窗端口。 |
| Android 9 没声音 | 系统版本限制，使用画面投屏即可。 |

## 开发与构建

技术栈为 Tauri 2、Rust、React、TypeScript 和 Vite。准备 Node.js 22.12+、Rust MSVC 工具链、Visual Studio C++ 构建工具与 Windows SDK，以及 WebView2。完整系统依赖参考 [Tauri 官方准备说明](https://v2.tauri.app/start/prerequisites/)，Rust 安装参考 [Rust 官方安装页](https://www.rust-lang.org/tools/install)。

在源码项目根目录使用 PowerShell：

```powershell
npm install
node scripts/prepare-core.mjs --from "C:\Tools\scrcpy-win64-v5.0.1"
npm run desktop
```

项目的桌面、打包和测试脚本会自动查找用户目录下的 Cargo。单独执行 Cargo 时，可为当前终端补充路径：

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo --version
```

单独预览前端：

```powershell
npm run dev
```

浏览器预览不能访问 adb 或启动投屏。界面演示数据仅供设计预览，不代表真实设备连接。

### 准备官方运行核心

从 [scrcpy 官方发布页](https://github.com/Genymobile/scrcpy/releases)下载 Windows x64 压缩包，完整解压后将目录传给 `--from`。脚本不下载发行包，不复制用户的 `.cmd`、`.bat`、`.vbs` 启动脚本。

`src-tauri/runtime/manifest.json` 记录来源目录、实际 scrcpy / adb 版本及每个文件的 SHA-256。首次来源为本机官方 scrcpy 5.0.1 解压目录，最终使用版本以清单为准。

```powershell
# 只校验并复用现有核心，不更新
npm run core:prepare

# 显式换用另一份完整官方核心
node scripts/prepare-core.mjs --from "D:\Tools\scrcpy-win64"
```

替换之前的核心会保留在 `.work` 备份目录中。运行核心缺失、哈希不符或版本验证失败时，脚本会停止并说明原因。

### 生成便携包

```powershell
npm run build
npm run package
```

打包脚本校验运行核心后，使用本地 Tauri CLI 执行 `build --no-bundle --features custom-protocol`，将界面嵌入主程序，并将完整运行核心和说明文件复制到暂存目录，再生成：

```text
outputs/
  Qzz-Mirror-0.2.1-win-x64/
  Qzz-Mirror-0.2.1-win-x64.zip
  Qzz-Mirror-0.2.1-win-x64.zip.sha256
```

`package-manifest.json` 保存主程序及运行核心清单的校验值；ZIP 的 SHA-256 单独提供。重复打包会将旧产物移入 `.work` 备份后发布新产物，不递归删除已有目录。构建失败时会保留暂存内容用于排查。

便携包包含更新日志及 `third-party-licenses` 许可文档。下载包的运行核心清单仅保留版本、上游来源和文件校验值，不包含开发机绝对路径；本地准备清单仍保留原始来源便于核对。

若默认输出目录中的程序仍在运行，可使用 `node scripts/package.mjs --output "outputs/releases/v0.2.1"` 输出到另一处项目内目录，保留正在运行的版本。

后端单元检查与隔离集成测试：

```powershell
npm run test:engine
```

界面布局与设置交互已在浏览器检查；0.2.1 更新品牌与设置图标。构建、隔离测试及历史真机记录见 [验收清单](docs/acceptance.md)。

## 许可与来源

自有代码许可见 [LICENSE](LICENSE)。原版 scrcpy 许可保留在 `runtime/LICENSE.txt`（源码准备后位于 `src-tauri/runtime/LICENSE.txt`）；第三方组件及来源见 [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)。Qzz Mirror 为独立控制面板，不代表 scrcpy 官方项目。
