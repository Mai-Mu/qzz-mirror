import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cli = path.join(root, 'node_modules/@tauri-apps/cli/tauri.js');
if (!existsSync(cli)) throw new Error('请先执行 npm install。');
const env = { ...process.env };
const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH';
env[pathKey] = `${path.join(os.homedir(), '.cargo/bin')}${path.delimiter}${env[pathKey] || ''}`;
const result = spawnSync(process.execPath, [cli, ...process.argv.slice(2)], {
  cwd: root, env, stdio: 'inherit', shell: false, windowsHide: true,
});
if (result.error) console.error(result.error.message);
process.exitCode = result.status ?? 1;
