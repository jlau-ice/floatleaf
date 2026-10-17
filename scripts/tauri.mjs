import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { rustEnvironment } from './runtime.mjs';

const require = createRequire(import.meta.url);
const cliDirectory = dirname(require.resolve('@tauri-apps/cli/package.json'));
const result = spawnSync(process.execPath, [join(cliDirectory, 'tauri.js'), ...process.argv.slice(2)], {
  stdio: 'inherit',
  env: rustEnvironment(),
});
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
