import { randomUUID } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { assertWorkspacePath, ensureWorkspaceDirectory, projectRoot, sha256, validateRuntime } from './prepare-core.mjs';

const fail = message => { throw new Error(message); };
const readJson = file => JSON.parse(readFileSync(file, 'utf8'));
const slash = value => value.replaceAll('\\', '/');

function relativeFile(root, relative) {
  if (!relative || path.isAbsolute(relative) || relative.split(/[\\/]/).some(part => part === '..' || part === '.')) fail(`非法文件路径：${relative}`);
  const file = path.resolve(root, relative);
  if (!file.startsWith(`${path.resolve(root)}${path.sep}`)) fail(`文件超出来源目录：${relative}`);
  assertWorkspacePath(file);
  if (!statSync(file).isFile()) fail(`不是普通文件：${relative}`);
  return file;
}

function assertX64(file) {
  const data = readFileSync(file);
  if (data.length < 64 || data.toString('ascii', 0, 2) !== 'MZ') fail('主程序不是有效的 Windows PE 文件。');
  const header = data.readUInt32LE(0x3c);
  if (header + 6 > data.length || data.readUInt32LE(header) !== 0x4550 || data.readUInt16LE(header + 4) !== 0x8664) fail('安装包要求 AMD64 主程序。');
}

function publish(source, destination, work) {
  assertWorkspacePath(source);
  assertWorkspacePath(destination, { mustExist: false });
  let backup;
  if (existsSync(destination)) {
    backup = path.join(work, `installer-backup-${randomUUID()}-${path.basename(destination)}`);
    renameSync(destination, backup);
  }
  try { renameSync(source, destination); }
  catch (error) {
    if (backup && !existsSync(destination)) renameSync(backup, destination);
    throw error;
  }
  if (backup) console.log(`旧产物已保留：${backup}`);
}

function main() {
  if (process.platform !== 'win32' || process.arch !== 'x64') fail('安装包需要在 Windows x64 环境生成。');
  const metadata = readJson(path.join(projectRoot, 'package.json'));
  const config = readJson(path.join(projectRoot, 'src-tauri/tauri.conf.json'));
  if (!/^\d+\.\d+\.\d+$/.test(metadata.version) || config.version !== metadata.version) fail('应用版本不一致或格式不正确。');
  const name = `Qzz-Mirror-${metadata.version}-win-x64`;
  const releaseSource = path.join(projectRoot, 'outputs/releases', `v${metadata.version}`, name);
  let source = existsSync(releaseSource) ? releaseSource : path.join(projectRoot, 'outputs', name);
  let output = path.join(projectRoot, 'outputs/installers', `v${metadata.version}`);
  const args = process.argv.slice(2);
  for (let index = 0; index < args.length; index += 2) {
    if (!args[index + 1] || !['--from', '--output'].includes(args[index])) fail('用法：node scripts/package-installer.mjs [--from "项目内便携目录"] [--output "项目内输出目录"]');
    if (args[index] === '--from') source = path.resolve(projectRoot, args[index + 1]);
    else output = path.resolve(projectRoot, args[index + 1]);
  }
  assertWorkspacePath(source);
  assertWorkspacePath(output, { mustExist: false });

  // Use the verified portable payload, never recursively package its user data.
  const manifest = readJson(relativeFile(source, 'package-manifest.json'));
  const executable = relativeFile(source, 'qzz-mirror.exe');
  const builtExecutable = path.join(projectRoot, 'src-tauri/target/release/qzz-mirror.exe');
  if (manifest.version !== metadata.version || manifest.platform !== 'windows-x64') fail('便携包与当前 Windows x64 版本不一致。');
  if (sha256(executable) !== manifest.executableSha256) fail('便携包主程序校验失败。');
  assertX64(executable);
  if (!existsSync(builtExecutable) || sha256(builtExecutable) !== manifest.executableSha256) fail('release 主程序与便携包不一致，请先用 npm run package 生成对应便携包，再用 --from 指向它。');
  const core = validateRuntime(path.join(source, 'runtime'));
  if (sha256(relativeFile(source, 'runtime/manifest.json')) !== manifest.runtimeManifestSha256 || core.version !== manifest.scrcpyVersion) fail('运行核心清单与便携包不一致。');
  if (Object.keys(core).some(key => !['schemaVersion', 'upstream', 'version', 'adbVersion', 'files'].includes(key))) fail('运行核心清单含本机元数据，请先生成可发布的便携包。');

  const documentation = ['README.md', 'CHANGELOG.md', 'LICENSE', 'THIRD-PARTY-NOTICES.md', 'docs/acceptance.md', 'docs/decisions.md'];
  const licenseIndex = 'third-party-licenses/checksums.sha256';
  const licenseFiles = readFileSync(relativeFile(source, licenseIndex), 'utf8').trim().split(/\r?\n/).map(line => {
    const match = line.match(/^([a-f0-9]{64})  (.+)$/);
    if (!match) fail('第三方许可索引格式错误。');
    const relative = `third-party-licenses/${match[2]}`;
    if (sha256(relativeFile(source, relative)) !== match[1]) fail(`第三方许可校验失败：${match[2]}`);
    return relative;
  });
  const files = ['package-manifest.json', ...documentation, 'runtime/manifest.json', ...core.files.map(file => `runtime/${file.name}`), licenseIndex, ...licenseFiles];
  if (new Set(files.map(file => file.toLowerCase())).size !== files.length) fail('安装资源清单含重复路径。');

  const work = ensureWorkspaceDirectory(path.join(projectRoot, '.work'));
  const stage = ensureWorkspaceDirectory(path.join(work, `installer-stage-${randomUUID()}`));
  const payload = ensureWorkspaceDirectory(path.join(stage, 'payload'));
  for (const relative of files) {
    const target = path.join(payload, relative);
    mkdirSync(path.dirname(target), { recursive: true });
    copyFileSync(relativeFile(source, relative), target);
  }
  const instructions = `Qzz Mirror · 轻镜 ${metadata.version} — Windows x64 安装版\r\n\r\n安装与使用\r\n1. 默认仅为当前 Windows 用户安装，通常不需要管理员权限。\r\n2. 通过开始菜单或桌面快捷方式打开轻镜，选择设备后开始投屏。\r\n3. 安装包包含 scrcpy ${core.version}、ADB 和第三方许可，不需要另行配置 ADB。\r\n4. 缺少 Microsoft Edge WebView2 Runtime 时，安装向导需要联网下载并安装。\r\n\r\n升级与卸载\r\n升级前，请右键旧版托盘图标选择“退出”；仅关闭面板会继续在托盘运行。\r\n可以在 Windows“设置 → 应用”中卸载，也可运行安装目录中的 uninstall.exe。\r\n应用设置通常保存在安装目录的 data 文件夹；目录不可写时使用当前用户的应用数据目录。\r\n卸载默认保留设置。安装目录中保留的 data 文件夹可在退出应用后自行备份或清理。\r\n便携版原有设置不会自动迁移；需要时请先退出两边程序，再复制原 data 文件夹到安装目录。\r\n\r\n项目：https://github.com/Mai-Mu/qzz-mirror\r\n`;
  writeFileSync(path.join(payload, 'INSTALLATION.txt'), instructions, 'utf8');
  files.push('INSTALLATION.txt');

  // CLI overlays deep-merge object maps. Explicit null entries remove ALL base
  // resource mappings before adding the staged, positively listed resources.
  const resources = {};
  if (config.bundle?.resources && !Array.isArray(config.bundle.resources)) {
    for (const key of Object.keys(config.bundle.resources)) resources[key] = null;
  }
  for (const relative of files) resources[slash(path.join(payload, relative))] = relative;
  const overlay = {
    bundle: {
      active: true, targets: ['nsis'], publisher: 'qzz',
      homepage: 'https://github.com/Mai-Mu/qzz-mirror',
      licenseFile: slash(path.join(payload, 'LICENSE')), resources,
      windows: {
        allowDowngrades: false,
        webviewInstallMode: { type: 'downloadBootstrapper', silent: true },
        nsis: {
          installMode: 'currentUser', languages: ['SimpChinese', 'English'],
          displayLanguageSelector: false, compression: 'lzma', installerIcon: 'icons/icon.ico',
        },
      },
    },
  };
  const overlayPath = path.join(stage, 'tauri.installer.conf.json');
  writeFileSync(overlayPath, `${JSON.stringify(overlay, null, 2)}\n`, 'utf8');
  const cargoBin = path.join(os.homedir(), '.cargo/bin');
  const env = { ...process.env };
  const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') || 'PATH';
  env[pathKey] = `${cargoBin}${path.delimiter}${env[pathKey] || ''}`;
  const cli = path.join(projectRoot, 'node_modules/@tauri-apps/cli/tauri.js');
  const result = spawnSync(process.execPath, [cli, 'bundle', '--bundles', 'nsis', '--features', 'custom-protocol', '--config', overlayPath, '--ci', '--no-binary-patching'], {
    cwd: projectRoot, env, stdio: 'inherit', shell: false, windowsHide: true,
  });
  if (result.error || result.status !== 0) fail(`安装包生成失败：${result.error?.message || `退出码 ${result.status}`}；暂存目录保留在 ${stage}`);
  if (sha256(builtExecutable) !== manifest.executableSha256) fail('打包工具修改了已验证主程序，请勿分发。');
  const bundleDirectory = path.join(projectRoot, 'src-tauri/target/release/bundle/nsis');
  const expectedName = `${config.productName}_${metadata.version}_x64-setup.exe`;
  if (!readdirSync(bundleDirectory).includes(expectedName)) fail(`未找到安装包：${expectedName}`);
  const installerName = `${name}-setup.exe`;
  const stagedInstaller = path.join(stage, installerName);
  copyFileSync(path.join(bundleDirectory, expectedName), stagedInstaller);
  const installerHash = sha256(stagedInstaller);
  writeFileSync(`${stagedInstaller}.sha256`, `${installerHash}  ${installerName}\n`, 'utf8');
  const metadataName = `${name}-installer-manifest.json`;
  writeFileSync(path.join(stage, metadataName), `${JSON.stringify({
    schemaVersion: 1, product: 'Qzz Mirror · 轻镜', version: metadata.version,
    platform: 'windows-x64', format: 'nsis', installMode: 'currentUser',
    webviewInstallMode: 'downloadBootstrapper', scrcpyVersion: core.version,
    executableSha256: manifest.executableSha256,
    installer: { name: installerName, size: statSync(stagedInstaller).size, sha256: installerHash },
    resources: files.map(relative => ({ name: relative, size: statSync(path.join(payload, relative)).size, sha256: sha256(path.join(payload, relative)) })),
  }, null, 2)}\n`, 'utf8');
  ensureWorkspaceDirectory(output);
  for (const file of [installerName, `${installerName}.sha256`, metadataName]) publish(path.join(stage, file), path.join(output, file), work);
  console.log(`Windows x64 安装包：${path.join(output, installerName)}`);
  console.log(`SHA-256：${path.join(output, `${installerName}.sha256`)}`);
  console.log(`安装资源暂存：${payload}`);
}

try { main(); }
catch (error) { console.error(`[package:installer] ${error.message}`); process.exitCode = 1; }
