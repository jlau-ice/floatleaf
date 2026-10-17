import { spawnSync } from 'node:child_process';
import { rustEnvironment } from './runtime.mjs';

const result = spawnSync('cargo', process.argv.slice(2), {
  stdio: 'inherit',
  env: rustEnvironment(),
});
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
