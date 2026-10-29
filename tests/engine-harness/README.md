# 引擎隔离测试（Windows）

测试 crate 直接导入生产代码 `engine.rs` 与 `models.rs`。模拟的 `adb.exe`、`scrcpy.exe` 仅使用本目录忽略的 `target/` 下隔离文件，不接触真实设备、ADB 服务、网络配置或现有投屏会话。

在源码项目根目录运行：

```powershell
npm run test:engine
```

或将 Rust 工具链加入 PATH 后分别执行：

```powershell
cargo test --manifest-path tests/engine-harness/Cargo.toml --lib --locked
cargo build --manifest-path tests/engine-harness/Cargo.toml --bin mock-runtime --bin mock-lifecycle --locked
& ./tests/engine-harness/target/debug/mock-lifecycle.exe
```

0.2.0 设置相关检查包含：

- 默认使用原始分辨率 `--max-size 0`；“最高”不传 `--max-fps`，手动整数范围为 1～240。
- 启动模拟子进程并记录实际 argv，检查默认不额外限帧与自定义 120 FPS；后端码率固定 `8M`。
- 保存设置和加载配置时拒绝非法帧率；从旧 `preset` 配置迁移后保留其他偏好与无线地址。

其余覆盖包括连接解析、按硬件标识合并设备、目标设备隔离、地址校验、Wi-Fi 路由、USB 辅助配置、投屏中拒绝破坏性配置、拔线后的同设备恢复（包括未经应用保存的已在线 Wi-Fi）、主动停止、正常关窗、首帧前或身份核验中取消启动，以及 Android 11 配对端口与连接端口分离和 mDNS 发现。无线启动还模拟地址被重新分配、设备无法提供硬件身份等情况。

断言失败时测试进程返回非零退出码，可在不连接手机的情况下重建并运行。隔离检查只能证明模拟条件下的后端行为，不能代替 0.2.0 原生界面、真实帧率、无线配对或投屏验收；实际记录见 [验收清单](../../docs/acceptance.md)。
