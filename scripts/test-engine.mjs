import { spawnSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cargoBin = path.join(os.homedir(), '.cargo/bin');
const cargo = path.join(cargoBin, process.platform === 'win32' ? 'cargo.exe' : 'cargo');
const env = { ...process.env };
const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH';
env[pathKey] = `${cargoBin}${path.delimiter}${env[pathKey] || ''}`;
function run(file, args) {
  const result = spawnSync(file, args, { cwd: root, env, stdio: 'inherit', shell: false, windowsHide: true });
  if (result.error || result.status !== 0) {
    console.error(result.error?.message || `测试进程退出：${result.status}`);
    process.exit(result.status || 1);
  }
}
if (process.platform !== 'win32') throw new Error('集成测试模拟 Windows 投屏组件，请在 Windows 上运行。');
const manifest = 'tests/engine-harness/Cargo.toml';
run(cargo, ['test', '--manifest-path', manifest, '--lib', '--locked']);
run(cargo, ['build', '--manifest-path', manifest, '--bin', 'mock-runtime', '--bin', 'mock-lifecycle', '--locked']);
run(path.join(root, 'tests/engine-harness/target/debug/mock-lifecycle.exe'), []);
