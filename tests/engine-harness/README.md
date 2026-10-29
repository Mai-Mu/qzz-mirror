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

导航检查使用可注入的记录 dispatcher，覆盖当前自有子进程 PID、无会话／旧 token／首帧未到／重连中的拒绝行为、投递失败，以及 USB 恢复为 Wi-Fi 后的新 token 和子进程。导航不再创建 ADB/input 进程，模拟 ADB 不接受按键命令。这里验证的是会话隔离和原生投递入口，不将记录 dispatcher 当作 Android 真实执行证明。

另运行 `node scripts/test-sdl-navigation.mjs`，可加载已准备的实际 SDL3.dll，在测试进程自己的隐藏 SDL 窗口中调用生产 dispatcher，检查左 Alt+s/b/h 的 SYSKEY 事件顺序、修饰状态、重复标志、文字消息与焦点/捕获/键盘状态。该检查不移动鼠标、不请求前台、不使用全局输入、不控制真实手机；耗时仅表示本地窗口投递及 SDL 事件消费，不代表 Android 端到端响应时间。结果写入 `.work/sdl-navigation-smoke.json`。

断言失败时测试进程返回非零退出码，可在不连接手机的情况下重建并运行。隔离检查只能证明模拟条件下的后端行为，不能代替 0.2.0 原生界面、真实帧率、无线配对或投屏验收；实际记录见 [验收清单](../../docs/acceptance.md)。
