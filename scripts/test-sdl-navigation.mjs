import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
if (process.platform !== 'win32') throw new Error('SDL 导航回归需要 Windows。');
if (!existsSync(path.join(root, 'src-tauri/runtime/SDL3.dll'))) {
  throw new Error('请先准备完整的官方 scrcpy 运行组件；测试只在自己创建的隐藏 SDL 窗口中运行。');
}
const cargoBin = path.join(os.homedir(), '.cargo/bin');
const env = { ...process.env };
const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH';
env[pathKey] = `${cargoBin}${path.delimiter}${env[pathKey] || ''}`;
function run(file, args) {
  const result = spawnSync(file, args, { cwd: root, env, stdio: 'inherit', shell: false, windowsHide: true });
  if (result.error || result.status !== 0) throw new Error(result.error?.message || `SDL 导航检查退出码：${result.status}`);
}
run(path.join(cargoBin, 'cargo.exe'), ['build', '--manifest-path', 'tests/engine-harness/Cargo.toml', '--bin', 'sdl-navigation', '--locked']);
run(path.join(root, 'tests/engine-harness/target/debug/sdl-navigation.exe'), []);
