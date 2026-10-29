import { createHash, randomUUID } from 'node:crypto';
import {
  copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync,
  readdirSync, realpathSync, renameSync, statSync, writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const projectRoot = realpathSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'));
export const runtimeDirectory = path.join(projectRoot, 'src-tauri', 'runtime');
const requiredFiles = ['scrcpy.exe', 'adb.exe', 'scrcpy-server', 'LICENSE.txt'];
const allowedFile = (name) => requiredFiles.includes(name)
  || /^[A-Za-z0-9][A-Za-z0-9._-]*\.dll$/i.test(name)
  || ['scrcpy.png', 'disconnected.png', 'NOTICE', 'NOTICE.txt'].includes(name);

export function sha256(file) {
  return createHash('sha256').update(readFileSync(file)).digest('hex');
}

function fail(message) {
  throw new Error(message);
}

export function assertWorkspacePath(target, { mustExist = true } = {}) {
  const absolute = path.resolve(target);
  const relative = path.relative(projectRoot, absolute);
  if (!relative || relative.startsWith(`..${path.sep}`) || relative === '..' || path.isAbsolute(relative)) {
    fail(`拒绝操作项目目录之外的路径：${absolute}`);
  }
  let ancestor = absolute;
  while (!existsSync(ancestor)) {
    const parent = path.dirname(ancestor);
    if (parent === ancestor) fail(`无法验证目标路径：${absolute}`);
    ancestor = parent;
  }
  const resolvedAncestor = realpathSync(ancestor);
  const resolvedRelative = path.relative(projectRoot, resolvedAncestor);
  if (resolvedRelative.startsWith(`..${path.sep}`) || resolvedRelative === '..' || path.isAbsolute(resolvedRelative)) {
    fail(`目标路径通过链接指向项目目录之外：${absolute}`);
  }
  if (mustExist && !existsSync(absolute)) fail(`路径不存在：${absolute}`);
  return absolute;
}

export function ensureWorkspaceDirectory(directory) {
  assertWorkspacePath(directory, { mustExist: false });
  mkdirSync(directory, { recursive: true });
  assertWorkspacePath(directory);
  if (!statSync(directory).isDirectory()) fail(`应为目录：${directory}`);
  return directory;
}

function checkRegularFile(file) {
  const info = lstatSync(file);
  if (!info.isFile() || info.isSymbolicLink()) fail(`只接受普通文件：${file}`);
  return info;
}

function readVersion(executable, args, directory) {
  if (process.platform !== 'win32') fail('此脚本准备 Windows 运行核心，请在 Windows 上执行。');
  const result = spawnSync(executable, args, {
    cwd: directory,
    shell: false,
    windowsHide: true,
    encoding: 'utf8',
    timeout: 15_000,
    maxBuffer: 2 * 1024 * 1024,
    env: {
      ...process.env,
      ADB: path.join(directory, 'adb.exe'),
      SCRCPY_SERVER_PATH: path.join(directory, 'scrcpy-server'),
    },
  });
  const output = `${result.stdout || ''}\n${result.stderr || ''}`.trim();
  if (result.error || result.status !== 0) {
    fail(`无法验证 ${path.basename(executable)}，请使用完整官方 Windows 压缩包。\n${result.error?.message || output || `退出码 ${result.status}`}`);
  }
  return output;
}

function versions(directory) {
  const scrcpyOutput = readVersion(path.join(directory, 'scrcpy.exe'), ['--version'], directory);
  const version = scrcpyOutput.match(/\bscrcpy\s+v?([0-9]+(?:\.[0-9]+)+(?:[-+][A-Za-z0-9.-]+)?)/i)?.[1];
  if (!version) fail(`无法从 scrcpy --version 识别版本：\n${scrcpyOutput}`);
  const adbOutput = readVersion(path.join(directory, 'adb.exe'), ['version'], directory);
  const adbVersion = adbOutput.match(/Android Debug Bridge version\s+([^\r\n]+)/i)?.[1];
  if (!adbVersion) fail(`无法从 adb version 识别版本：\n${adbOutput}`);
  return { version, adbVersion, scrcpyVersionOutput: scrcpyOutput, adbVersionOutput: adbOutput };
}

export function validateRuntime(directory = runtimeDirectory, { executeVersions = true } = {}) {
  if (!existsSync(directory)) fail(`缺少运行核心：${directory}\n请从 https://github.com/Genymobile/scrcpy/releases 下载并解压官方 Windows 64 位压缩包，然后执行：\nnpm run core:prepare -- --from "解压后的 scrcpy 目录"`);
  const manifestPath = path.join(directory, 'manifest.json');
  if (!existsSync(manifestPath)) fail(`缺少运行核心清单：${manifestPath}\n请通过 npm run core:prepare -- --from "官方 scrcpy 目录" 重新准备。`);
  let manifest;
  try { manifest = JSON.parse(readFileSync(manifestPath, 'utf8')); }
  catch { fail(`运行核心清单无法读取：${manifestPath}`); }
  if (manifest.schemaVersion !== 1 || typeof manifest.version !== 'string' || !Array.isArray(manifest.files)) {
    fail('运行核心清单格式不正确，请显式指定 --from 重新准备。');
  }
  const seen = new Set();
  for (const entry of manifest.files) {
    if (!entry || typeof entry.name !== 'string' || path.basename(entry.name) !== entry.name
      || !allowedFile(entry.name) || !/^[a-f0-9]{64}$/.test(entry.sha256 || '')
      || !Number.isSafeInteger(entry.size) || entry.size < 1) fail('运行核心清单包含不合法文件项。');
    const normalized = entry.name.toLowerCase();
    if (seen.has(normalized)) fail(`运行核心清单包含重复文件：${entry.name}`);
    seen.add(normalized);
    const file = path.join(directory, entry.name);
    if (!existsSync(file)) fail(`运行核心文件缺失：${entry.name}`);
    const info = checkRegularFile(file);
    if (info.size !== entry.size || sha256(file) !== entry.sha256) {
      fail(`运行核心文件校验失败：${entry.name}\n脚本不会自动更新或覆盖，请核对来源后显式使用 --from。`);
    }
  }
  for (const name of requiredFiles) if (!seen.has(name.toLowerCase())) fail(`运行核心清单缺少：${name}`);
  if (![...seen].some((name) => name.endsWith('.dll'))) fail('运行核心中未包含官方 DLL。');
  for (const name of readdirSync(directory)) {
    if (name !== 'manifest.json' && !seen.has(name.toLowerCase())) {
      fail(`运行核心包含清单外文件：${name}\n请核对来源后显式使用 --from 重新准备。`);
    }
  }
  if (executeVersions) {
    const actual = versions(directory);
    if (actual.version !== manifest.version || actual.adbVersion !== manifest.adbVersion) fail('运行核心版本与清单不一致。');
  }
  return manifest;
}

function prepare(sourceArgument) {
  const source = realpathSync(path.resolve(sourceArgument));
  if (!statSync(source).isDirectory()) fail(`来源不是目录：${source}`);
  if (source === runtimeDirectory || source.startsWith(`${runtimeDirectory}${path.sep}`)) fail('--from 必须指向官方压缩包解压目录，不能指向目标 runtime 本身。');
  const names = readdirSync(source).filter(allowedFile).sort((a, b) => a.localeCompare(b, 'en'));
  for (const name of requiredFiles) if (!names.includes(name)) fail(`来源缺少 ${name}：${source}\n请选择同时包含 scrcpy.exe、adb.exe、scrcpy-server、DLL 和 LICENSE.txt 的完整解压目录。`);
  if (!names.some((name) => name.toLowerCase().endsWith('.dll'))) fail('来源缺少 DLL，请使用完整官方压缩包。');
  for (const name of names) checkRegularFile(path.join(source, name));
  const detected = versions(source);
  const work = ensureWorkspaceDirectory(path.join(projectRoot, '.work'));
  const staging = ensureWorkspaceDirectory(path.join(work, `runtime-stage-${randomUUID()}`));
  for (const name of names) copyFileSync(path.join(source, name), path.join(staging, name));
  const manifest = {
    schemaVersion: 1,
    source,
    upstream: 'https://github.com/Genymobile/scrcpy',
    preparedAt: new Date().toISOString(),
    ...detected,
    files: names.map((name) => ({ name, size: statSync(path.join(staging, name)).size, sha256: sha256(path.join(staging, name)) })),
  };
  writeFileSync(path.join(staging, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  validateRuntime(staging);
  ensureWorkspaceDirectory(path.dirname(runtimeDirectory));
  let backup;
  if (existsSync(runtimeDirectory)) {
    assertWorkspacePath(runtimeDirectory);
    backup = path.join(work, `runtime-backup-${randomUUID()}`);
    assertWorkspacePath(backup, { mustExist: false });
    renameSync(runtimeDirectory, backup);
  }
  try {
    assertWorkspacePath(staging);
    assertWorkspacePath(runtimeDirectory, { mustExist: false });
    renameSync(staging, runtimeDirectory);
  } catch (error) {
    if (backup && !existsSync(runtimeDirectory)) renameSync(backup, runtimeDirectory);
    throw error;
  }
  console.log(`已准备 scrcpy ${manifest.version} / adb ${manifest.adbVersion}，共 ${manifest.files.length} 个文件。`);
  console.log(`运行核心：${runtimeDirectory}`);
  if (backup) console.log(`上一版已保留：${backup}`);
}

function main() {
  const args = process.argv.slice(2);
  if (args.length === 1 && ['--help', '-h'].includes(args[0])) {
    console.log('用法：\n  npm run core:prepare\n  npm run core:prepare -- --from "官方 scrcpy Windows 64 位解压目录"\n\n未指定 --from 时只校验并复用现有 runtime，不下载、不更新。');
    return;
  }
  if (args.length === 0) {
    const manifest = validateRuntime();
    console.log(`已校验并复用 scrcpy ${manifest.version}，未更新运行核心。`);
    return;
  }
  if (args.length !== 2 || args[0] !== '--from') fail('参数不正确。使用 --from "官方 scrcpy 解压目录"，或 --help 查看说明。');
  prepare(args[1]);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); }
  catch (error) { console.error(`[core:prepare] ${error.message}`); process.exitCode = 1; }
}
