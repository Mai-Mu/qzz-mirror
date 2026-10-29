# 第三方许可与来源记录

本目录随 Qzz Mirror 0.2.1 分发，保存按当前实际依赖整理的许可/版权通知。上游许可文本按原字节复制；`checksums.sha256` 记录本目录文件的 SHA-256。本文不等同于完整合规认证。

## 投屏运行组件

| 组件 | 本包实际版本或证据 | 许可资料与官方来源 |
| --- | --- | --- |
| scrcpy / scrcpy-server | 5.0.1；保留官方运行文件 | [v5.0.1 LICENSE](https://github.com/Genymobile/scrcpy/blob/v5.0.1/LICENSE) → `scrcpy/LICENSE.txt`。与 `runtime/LICENSE.txt` 原文一致。 |
| adb / AdbWinApi.dll / AdbWinUsbApi.dll | Platform Tools 37.0.1，adb 协议版本 1.0.41 | [Google 原始 SDK ZIP](https://dl.google.com/android/repository/platform-tools_r37.0.1-win.zip) → `adb/NOTICE.txt`、`adb/source.properties.txt`。SDK ZIP 已按官方 scrcpy 脚本给出的 SHA-256 校验；三个运行文件 SHA-256 均与本包相同，证据见 `adb/SOURCE.json`。 |
| SDL3.dll | `SDL_GetVersion` = 3004018；scrcpy 编译/链接均为 3.4.18 | [release-3.4.18 LICENSE.txt](https://github.com/libsdl-org/SDL/blob/release-3.4.18/LICENSE.txt) 和同 tag `CREDITS.md` → `sdl/`。 |
| FFmpeg DLL | avcodec / avformat 63.1.102，avutil 61.1.102，swresample 7.1.102；同版 scrcpy 构建脚本指定 FFmpeg 9.0.2 | [n9.0.2 LGPL 全文](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/COPYING.LGPLv2.1)、[LICENSE.md](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/LICENSE.md) → `ffmpeg/`。四个 DLL 的 `*_license()` 均返回 “LGPL version 2.1 or later”，配置未出现 `--enable-gpl` / `--enable-nonfree`；导出函数原始结果与 DLL 哈希见 `ffmpeg/BINARY-INSPECTION.json`。 |
| libusb-1.0.dll | 1.0.30；scrcpy 链接版本与官方构建脚本一致 | [v1.0.30 COPYING](https://github.com/libusb/libusb/blob/v1.0.30/COPYING)、同 tag `AUTHORS` → `libusb/`。 |
| FFmpeg 中的 dav1d | 1.5.4；来自 scrcpy 5.0.1 官方依赖构建脚本，FFmpeg 实际配置含 `--enable-libdav1d` | [1.5.4 COPYING](https://github.com/videolan/dav1d/blob/1.5.4/COPYING) → `dav1d/COPYING.txt`。未单独从静态嵌入的库导出精确版本。 |
| FFmpeg 中的 zlib | 实际 avcodec、avformat DLL 含 `inflate 1.3.1 Copyright 1995-2024 Mark Adler` 标识 | [v1.3.1 LICENSE](https://github.com/madler/zlib/blob/v1.3.1/LICENSE) → `zlib/LICENSE.txt`。 |

`scrcpy/build-sources/` 的 `.txt` 文件是 [scrcpy v5.0.1/app/deps](https://github.com/Genymobile/scrcpy/tree/v5.0.1/app/deps) 下原始构建脚本的逐字节副本，仅为避免误执行而增加 `.txt` 后缀。保留 `_init`、`ffmpeg.sh`、`sdl.sh`、`libusb.sh`、`dav1d.sh`、`adb_windows.sh`；它们记录上游依赖版本、源码地址、原始归档 SHA-256 与构建选项。

## 控制面板依赖

- `react/`：React / react-dom 19.3.1、scheduler 0.28.0。许可直接来自相同版本已安装包；见 `react/SOURCES.md` 与 `react/FILES.json`。
- `tauri/`：Tauri JS API 2.12.4、tauri 2.12.5、tauri-plugin-single-instance 2.5.4 及 Windows x86-64 普通运行依赖图中的 147 个 Rust crate。许可/NOTICE/SPDX 来自当前锁文件版本的 crate 包；三个本地包未附顶层许可的情况已从其 `.cargo_vcs_info.json` 指定的官方仓库精确提交补齐，见 `tauri/SOURCES.md`、`tauri/FILES.json`、`tauri/DEPENDENCIES.json`。
- 文档中 `node_modules/<包>/`、`cargo-registry/<crate>-<version>/` 是相对来源标识，不包含开发机用户名或绝对目录。Rust 依赖图排除了构建依赖和过程宏；列入普通依赖图也不证明每个 crate 的代码最终均被链接进 EXE。

## LGPL 原始源码附件

同一 GitHub Release 页提供 `Qzz-Mirror-0.2.1-third-party-sources.zip` 与 `.sha256`。源码附件保留两个官方原始归档，不对源代码重排或修改：

| 原始归档 | 官方下载地址 | 官方 scrcpy 5.0.1 构建脚本指定的 SHA-256 |
| --- | --- | --- |
| `ffmpeg-9.0.2.tar.xz` | [FFmpeg release source](https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz) | `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e` |
| `libusb-1.0.30.tar.gz` | [libusb v1.0.30 source](https://github.com/libusb/libusb/archive/refs/tags/v1.0.30.tar.gz) | `2ae28adb0bb9558c86135c4e1c11b320b0805461e207a64a6e520a114094bf07` |

附件内同时保留上述官方构建脚本、实际 FFmpeg DLL 的许可/版本/配置与哈希记录，并给出逐文件校验清单。它用于让接收者取得本次标识的原始源码和构建资料，不是自行替上游作多年书面提供承诺。

## 核验范围与边界

1. scrcpy 官方脚本指定的版本与本包导出函数/版本输出相符；ADB 三个文件还与 Google 原始 SDK 包的文件逐字节匹配。未在本次工作中完整重建 scrcpy、FFmpeg 或 libusb，因此没有宣称可逐字节复现官方发行二进制。
2. FFmpeg 实际 DLL 报告的 LGPL 版本与配置均已保存；相同项目的其他构建可能启用 GPL、nonfree 或不同静态依赖，不能套用本包结论。
3. 已收集当前范围的主要原始通知、依赖包许可与 LGPL 源码附件资料；没有完成全部文件级版权、Rust 标准库、MinGW/GCC 等编译工具链、Windows 原生 SDK、WebView2 loader/系统运行环境等条款的完整审计。WebView2 系统运行环境本身没有随本便携包分发。
4. 许可文本、源码可获得性、具体二进制构建方式与其他适用条款应共同核对；不能仅凭本目录存在便宣称所有潜在分发义务都已满足。
