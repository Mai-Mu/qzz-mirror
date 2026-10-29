# 第三方组件与来源

Qzz Mirror · 轻镜通过独立进程调用官方 scrcpy 与 adb。下表记录 v0.2.1 采用的上游来源及许可位置。运行核心保留原版许可，控制面板依赖的精确版本记录在锁文件中。随包的 `third-party-licenses/` 保存实际许可、版权通知、来源记录与 SHA-256 索引；总览见 `third-party-licenses/SOURCE.md`。

## 随运行核心保留的组件

| 组件 / 文件 | 来源与许可资料 | 本项目处理方式 |
| --- | --- | --- |
| scrcpy / scrcpy-server | [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)，[Apache License 2.0](https://github.com/Genymobile/scrcpy/blob/v5.0.1/LICENSE) | 使用官方 Windows 文件，不修改投屏核心；原许可按原样复制为 `runtime/LICENSE.txt`。 |
| adb、AdbWinApi.dll、AdbWinUsbApi.dll | Android SDK Platform Tools 37.0.1 / Android Open Source Project；[Google 官方 SDK 包](https://dl.google.com/android/repository/platform-tools_r37.0.1-win.zip)与 [Platform Tools 官方资料](https://developer.android.com/tools/releases/platform-tools) | 三个文件均与该官方 SDK 包逐字节 SHA-256 一致；原包完整通知保存在 `third-party-licenses/adb/NOTICE.txt`，来源校验见同目录 `SOURCE.json`。 |
| SDL3.dll | [Simple DirectMedia Layer 3.4.18](https://github.com/libsdl-org/SDL/tree/release-3.4.18)，zlib 许可 | 版本经导出函数与 `scrcpy --version` 核对；原文及作者资料保存在 `third-party-licenses/sdl/`。 |
| avcodec、avformat、avutil、swresample DLL | [FFmpeg 9.0.2](https://ffmpeg.org/)，[官方许可说明](https://ffmpeg.org/legal.html) | 四个实际 DLL 的导出函数均报告 LGPL 2.1 或以后版本；构建参数未启用 GPL/nonfree。许可全文与实际版本、配置、二进制 SHA-256 保存在 `third-party-licenses/ffmpeg/`。此结论只针对本包实际文件，其他 FFmpeg 构建可能适用不同许可。 |
| libusb-1.0.dll | [libusb 1.0.30](https://github.com/libusb/libusb/tree/v1.0.30)，LGPL 2.1 或以后版本 | 保留官方核心中的 DLL；许可全文及作者资料保存在 `third-party-licenses/libusb/`。 |
| FFmpeg 内使用的 dav1d / zlib | [dav1d 1.5.4](https://github.com/videolan/dav1d/tree/1.5.4)（BSD-2-Clause）与 [zlib 1.3.1](https://github.com/madler/zlib/tree/v1.3.1)（zlib 许可） | dav1d 版本来自同版 scrcpy 构建脚本；实际 FFmpeg 二进制含 zlib inflate 1.3.1 标识。相应原文保存在 `third-party-licenses/dav1d/` 与 `third-party-licenses/zlib/`。 |
| scrcpy.png、disconnected.png（如存在） | 官方 scrcpy Windows 压缩包 | 原样保留作为 scrcpy 运行资源，不作为轻镜自有标识。 |

本机最初来源为官方 `scrcpy-win64-v5.0.1` 解压目录。`scripts/prepare-core.mjs` 会执行 `scrcpy --version` 与 `adb version`，记录来源路径、版本输出和复制文件的 SHA-256，用于追踪实际选用的运行文件。

原版 scrcpy 许可文件包含以下权利人通知，随 `runtime/LICENSE.txt` 保留：Genymobile（2018）、Romain Vimont（2018–2026）。这些名称不表示上游为 Qzz Mirror 提供背书。

## 控制面板依赖

前端使用 React / react-dom 19.3.1、scheduler 0.28.0（MIT）与 Tauri JavaScript API 2.12.4；桌面层使用 tauri 2.12.5、tauri-plugin-single-instance 2.5.4 及 Rust 生态依赖。Tauri 主要项目采用 MIT / Apache 2.0 双许可，具体 crate 和间接依赖以仓库中 `Cargo.lock`、npm 锁文件及对应版本的许可文件为准。`third-party-licenses/react/` 保存已安装 React 包的原始许可；`third-party-licenses/tauri/` 保存 Tauri API 与 Windows 普通运行依赖图中 147 个 Rust crate 的许可/NOTICE/SPDX 文本及逐项来源清单。构建工具如 TypeScript、Vite 与 Tauri CLI 不等同于随包独立分发的运行程序。

Windows WebView2 是应用使用的系统运行环境，遵循 Microsoft 提供的相关条款；本便携包不包含 WebView2 安装程序。

## 原始源码与构建资料

本版本 GitHub Release 同页提供 `Qzz-Mirror-0.2.1-third-party-sources.zip` 及其 `.sha256` 校验文件，包含 FFmpeg 9.0.2、libusb 1.0.30 的原始完整源码压缩包、对应官方 scrcpy 构建脚本以及本包实测的 FFmpeg 许可/配置记录。原始源码包使用同版 scrcpy 官方脚本记录的 SHA-256 校验；具体来源与对应关系记录在附件内的清单中。控制面板源码则由 Qzz Mirror 仓库提供。

这些资料用于保留许可通知和提高源码可获得性，不是全软件、工具链或全部文件级权利归属的完整审计，也不表示已证明可逐字节重建官方二进制；未在此作额外的多年书面提供承诺。具体范围和仍未核实的部分见 `third-party-licenses/SOURCE.md`。

上游组件各自遵循其原有许可证；控制面板自有代码采用 Apache 2.0。
