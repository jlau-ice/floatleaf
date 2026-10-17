import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, join } from 'node:path';

// rustup owns the toolchain. Add its shims only to child processes, without
// modifying the user's shell profile. Respect a custom CARGO_HOME.
export function rustEnvironment() {
  const cargoBin = join(process.env.CARGO_HOME || join(homedir(), '.cargo'), 'bin');
  const environment = { ...process.env };
  const pathKey = Object.keys(environment).find((key) => key.toLowerCase() === 'path') || 'PATH';
  if (existsSync(cargoBin)) {
    environment[pathKey] = [cargoBin, environment[pathKey] || ''].join(delimiter);
  }
  return environment;
}
