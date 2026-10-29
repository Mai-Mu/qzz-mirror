import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
if (process.platform !== 'win32') throw new Error('The navigation smoke harness requires Windows.');
const output = path.join(root, '.work');
mkdirSync(output, { recursive: true });
const cargoBin = path.join(os.homedir(), '.cargo', 'bin');
const cargo = path.join(cargoBin, 'cargo.exe');
const env = { ...process.env, QZZ_NAV_OUTPUT_DIR: output };
const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH';
env[pathKey] = `${cargoBin}${path.delimiter}${env[pathKey] || ''}`;
const manifest = path.join(root, 'tests/navigation-harness/Cargo.toml');

function run(file, args) {
  const result = spawnSync(file, args, { cwd: root, env, stdio: 'inherit', shell: false, windowsHide: true, timeout: 120_000 });
  if (result.error || result.status !== 0) {
    console.error(result.error?.message || `Navigation smoke failed: ${result.status}`);
    process.exit(result.status || 1);
  }
}

// The binary creates its own isolated mock SDL processes and a real toolbar.
// It never sends ADB commands, global input, cursor movement or activation calls.
const cargoArguments = ['--manifest-path', manifest, '--offline', '--locked'];
// Production module unit tests exercise edge/fullscreen/negative-coordinate
// placement without moving the cursor or showing extra native windows.
run(cargo, ['test', ...cargoArguments]);
run(cargo, ['build', ...cargoArguments]);
run(path.join(root, 'tests/navigation-harness/target/debug/qzz-navigation-harness.exe'), []);

const bmp = path.join(output, 'navigation-bar-smoke.bmp');
const png = path.join(output, 'navigation-bar-smoke.png');
if (existsSync(bmp)) {
  const code = 'from PIL import Image; import sys; Image.open(sys.argv[1]).convert("RGB").save(sys.argv[2])';
  const converted = spawnSync('python', ['-c', code, bmp, png], { cwd: root, env, shell: false, windowsHide: true, timeout: 15_000 });
  if (converted.status === 0) console.log(`Own-toolbar preview: ${png}`);
  else console.log(`Own-toolbar preview retained as BMP: ${bmp}`);
}
const resized = path.join(output, 'navigation-bar-resize.bmp');
if (existsSync(resized)) {
  const code = 'from PIL import Image; import sys; Image.open(sys.argv[1]).convert("RGB").save(sys.argv[2])';
  spawnSync('python', ['-c', code, resized, path.join(output, 'navigation-bar-resize.png')], { cwd: root, env, shell: false, windowsHide: true, timeout: 15_000 });
}
console.log(`Report: ${path.join(output, 'navigation-smoke.json')}`);
