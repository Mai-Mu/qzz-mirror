import { randomUUID } from 'node:crypto';
import { copyFileSync, cpSync, existsSync, readFileSync, renameSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import {
  assertWorkspacePath, ensureWorkspaceDirectory, projectRoot,
  runtimeDirectory, sha256, validateRuntime,
} from './prepare-core.mjs';

function fail(message) { throw new Error(message); }

function renameWithRetry(source, destination) {
  // Windows can retain a short-lived scan/loader handle after verifying EXEs.
  // Keep the staged package intact and retry only transient sharing failures.
  for (let attempt = 0; ; attempt++) {
    try { renameSync(source, destination); return; }
    catch (error) {
      if (attempt >= 24 || !['EPERM', 'EACCES', 'EBUSY', 'ENOTEMPTY'].includes(error.code)) throw error;
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 200);
    }
  }
}

function execute(file, args, env) {
  const result = spawnSync(file, args, {
    cwd: projectRoot,
    env,
    shell: false,
    windowsHide: true,
    stdio: 'inherit',
  });
  if (result.error || result.status !== 0) fail(`${path.basename(file)} 执行失败：${result.error?.message || `退出码 ${result.status}`}`);
}

function publish(source, destination, workDirectory) {
  assertWorkspacePath(source);
  assertWorkspacePath(destination, { mustExist: false });
  let backup;
  if (existsSync(destination)) {
    assertWorkspacePath(destination);
    backup = path.join(workDirectory, `package-backup-${randomUUID()}-${path.basename(destination)}`);
    assertWorkspacePath(backup, { mustExist: false });
    renameWithRetry(destination, backup);
  }
  try {
    try { renameWithRetry(source, destination); }
    catch (error) {
      if (!existsSync(destination) && ['EPERM', 'EACCES', 'EBUSY'].includes(error.code)) {
        // A scanner may keep a directory open. Publish an exact copy while
        // retaining the complete staged source; never delete a user's files.
        cpSync(source, destination, { recursive: true, errorOnExist: true, force: false });
        console.log(`Windows 文件占用，已复制交付并保留暂存：${destination}`);
      } else throw error;
    }
  }
  catch (error) {
    if (backup && !existsSync(destination)) renameWithRetry(backup, destination);
    throw error;
  }
  if (backup) console.log(`上一份产物已保留：${backup}`);
}

function main() {
  if (process.platform !== 'win32' || process.arch !== 'x64') fail('首版打包目标为 Windows x64，请在对应 Windows 环境执行。');
  const args = process.argv.slice(2);
  if (args.length !== 0 && (args.length !== 2 || args[0] !== '--output')) fail('用法：node scripts/package.mjs [--output "项目内输出目录"]');
  const outputPath = args.length === 2 ? path.resolve(projectRoot, args[1]) : path.join(projectRoot, 'outputs');
  assertWorkspacePath(outputPath, { mustExist: false });
  const manifest = validateRuntime();
  const metadata = JSON.parse(readFileSync(path.join(projectRoot, 'package.json'), 'utf8'));
  if (!/^\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$/.test(metadata.version || '')) fail('package.json 中的版本号格式不正确。');
  if (!existsSync(path.join(projectRoot, 'dist', 'index.html'))) fail('尚未构建前端，请先运行 npm run build，再运行 npm run package。');
  const tauriCli = path.join(projectRoot, 'node_modules', '@tauri-apps', 'cli', 'tauri.js');
  if (!existsSync(tauriCli)) fail('缺少本地 Tauri CLI，请先运行 npm install。');
  const cargoBin = path.join(os.homedir(), '.cargo', 'bin');
  const env = { ...process.env, PATH: `${cargoBin}${path.delimiter}${process.env.PATH || process.env.Path || ''}` };
  // Directly execute the installed CLI with Node: equivalent to npx tauri build --no-bundle,
  // without routing user-controlled paths through npm.cmd or a command shell.
  execute(process.execPath, [tauriCli, 'build', '--no-bundle', '--features', 'custom-protocol'], env);
  const executable = path.join(projectRoot, 'src-tauri', 'target', 'release', 'qzz-mirror.exe');
  if (!existsSync(executable) || !statSync(executable).isFile()) fail(`Tauri 构建后未找到主程序：${executable}`);
  const documentation = ['README.md', 'CHANGELOG.md', 'LICENSE', 'THIRD-PARTY-NOTICES.md', 'docs/acceptance.md', 'docs/decisions.md'];
  for (const relative of documentation) if (!existsSync(path.join(projectRoot, relative))) fail(`缺少交付文档：${relative}`);
  const work = ensureWorkspaceDirectory(path.join(projectRoot, '.work'));
  const stagingRoot = ensureWorkspaceDirectory(path.join(work, `package-stage-${randomUUID()}`));
  const productName = `Qzz-Mirror-${metadata.version}-win-x64`;
  const staging = ensureWorkspaceDirectory(path.join(stagingRoot, productName));
  copyFileSync(executable, path.join(staging, 'qzz-mirror.exe'));
  const runtimeTarget = ensureWorkspaceDirectory(path.join(staging, 'runtime'));
  for (const entry of manifest.files) copyFileSync(path.join(runtimeDirectory, entry.name), path.join(runtimeTarget, entry.name));
  // Publish only reproducibility metadata, keeping local paths and host details
  // in the development manifest rather than the downloadable archive.
  const portableManifest = {
    schemaVersion: manifest.schemaVersion,
    upstream: manifest.upstream,
    version: manifest.version,
    adbVersion: manifest.adbVersion,
    files: manifest.files.map(({ name, size, sha256 }) => ({ name, size, sha256 })),
  };
  writeFileSync(path.join(runtimeTarget, 'manifest.json'), `${JSON.stringify(portableManifest, null, 2)}\n`, 'utf8');
  const licenses = path.join(projectRoot, 'third-party-licenses');
  if (!existsSync(licenses) || !statSync(licenses).isDirectory()) fail('缺少第三方许可文档目录：third-party-licenses');
  cpSync(licenses, path.join(staging, 'third-party-licenses'), { recursive: true, errorOnExist: true, force: false });
  for (const relative of documentation) {
    const target = path.join(staging, relative);
    ensureWorkspaceDirectory(path.dirname(target));
    copyFileSync(path.join(projectRoot, relative), target);
  }
  writeFileSync(path.join(staging, 'package-manifest.json'), `${JSON.stringify({
    schemaVersion: 1,
    product: 'Qzz Mirror · 轻镜',
    version: metadata.version,
    platform: 'windows-x64',
    packagedAt: new Date().toISOString(),
    scrcpyVersion: manifest.version,
    executableSha256: sha256(path.join(staging, 'qzz-mirror.exe')),
    runtimeManifestSha256: sha256(path.join(runtimeTarget, 'manifest.json')),
  }, null, 2)}\n`, 'utf8');
  validateRuntime(runtimeTarget);
  const zipName = `${productName}.zip`;
  const zipStaging = path.join(stagingRoot, zipName);
  const powershell = path.join(process.env.SystemRoot || 'C:\\Windows', 'System32', 'WindowsPowerShell', 'v1.0', 'powershell.exe');
  const zipScript = "$ErrorActionPreference = 'Stop'; Compress-Archive -LiteralPath $env:QZZ_PACKAGE_SOURCE -DestinationPath $env:QZZ_PACKAGE_ZIP -CompressionLevel Optimal";
  execute(powershell, ['-NoLogo', '-NoProfile', '-NonInteractive', '-Command', zipScript], {
    ...env,
    QZZ_PACKAGE_SOURCE: staging,
    QZZ_PACKAGE_ZIP: zipStaging,
  });
  if (!existsSync(zipStaging) || statSync(zipStaging).size === 0) fail('ZIP 未成功生成，暂存目录已保留。');
  const checksumStaging = `${zipStaging}.sha256`;
  writeFileSync(checksumStaging, `${sha256(zipStaging)}  ${zipName}\n`, 'utf8');
  const output = ensureWorkspaceDirectory(outputPath);
  publish(staging, path.join(output, productName), work);
  publish(zipStaging, path.join(output, zipName), work);
  publish(checksumStaging, path.join(output, `${zipName}.sha256`), work);
  console.log(`便携目录：${path.join(output, productName)}`);
  console.log(`压缩包：${path.join(output, zipName)}`);
  console.log(`SHA-256：${path.join(output, `${zipName}.sha256`)}`);
}

try { main(); }
catch (error) { console.error(`[package] ${error.message}`); process.exitCode = 1; }
