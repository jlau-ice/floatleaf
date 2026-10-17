import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { rustEnvironment } from './runtime.mjs';

const environment = rustEnvironment();
let failed = false;
console.log(`FloatLeaf environment | ${process.platform} ${process.arch}`);
console.log(`node: ${process.version}`);
const [major, minor, patch] = process.versions.node.split('.').map(Number);
if (major !== 22 || minor < 22 || (minor === 22 && patch < 3)) {
  console.error('Expected Node >=22.22.3 <23; run nvm use.');
  failed = true;
}
const toolchain = readFileSync(new URL('../rust-toolchain.toml', import.meta.url), 'utf8');
console.log(`project toolchain: ${toolchain.match(/channel\s*=\s*"([^"]+)"/)?.[1]}`);
for (const [command, args] of [
  ['rustup', ['show', 'active-toolchain']],
  ['rustc', ['--version']],
  ['cargo', ['--version']],
  ['cargo', ['fmt', '--version']],
  ['cargo', ['clippy', '--version']],
  ['git', ['--version']],
]) {
  const result = spawnSync(command, args, { encoding: 'utf8', env: environment });
  console.log(`${command}: ${(result.stdout || result.stderr || result.error?.message || '').trim()}`);
  if (result.status !== 0) failed = true;
}
if (process.platform === 'darwin') {
  const sdk = spawnSync('xcrun', ['--show-sdk-path'], { encoding: 'utf8' });
  console.log(`macOS SDK: ${(sdk.stdout || sdk.stderr || '').trim()}`);
  if (sdk.status !== 0) failed = true;
}
if (process.platform === 'win32') {
  console.log('Also verify Visual Studio C++ build tools and WebView2 by building the desktop app.');
}
console.log('This checks tool availability; desktop behavior still requires a native build and manual validation.');
process.exitCode = failed ? 1 : 0;
