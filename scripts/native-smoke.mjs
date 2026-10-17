// Opt-in debug-only native window checks. Not a physical keyboard/mouse test.
import { readFile, writeFile, rename } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const directory = resolve(process.argv[2] || '.local/p0');
async function command(value, predicate) {
  const before = await readFile(join(directory, 'status.json'), 'utf8').catch(() => '');
  await writeFile(join(directory, 'command.tmp'), value);
  await rename(join(directory, 'command.tmp'), join(directory, 'command.txt'));
  for (let i = 0; i < 100; i++) {
    await delay(20);
    const raw = await readFile(join(directory, 'status.json'), 'utf8').catch(() => '');
    if (!raw || raw === before) continue;
    const current = JSON.parse(raw);
    if (predicate(current)) return current;
  }
  throw Error(`Native action timed out: ${value}`);
}
await command('toggle', (s) => s.state.mode === 'toggle' && !s.visible);
const initial = await command('hold', (s) => s.state.mode === 'hold' && !s.visible);
if (initial.state.error) throw Error(initial.state.error);
const samples = [];
for (let i = 0; i < 100; i++) {
  const shown = await command('pinned', (s) => s.visible && s.state.mode === 'pinned');
  if (
    shown.native.keyWindow ||
    !shown.native.clickthrough ||
    shown.native.opaque ||
    shown.native.level < 1
  )
    throw Error('Native window invariant failed');
  if (shown.native.frontmostPid !== initial.native.frontmostPid)
    throw Error('Foreground application changed; inspect whether this was external input');
  const hidden = await command('hold', (s) => !s.visible && s.state.mode === 'hold');
  samples.push({
    showRevision: shown.state.revision,
    hideRevision: hidden.state.revision,
    frontmostPid: shown.native.frontmostPid,
  });
}
await command('hide', (s) => s.state.emergencyLatched);
const suppressed = await command('pinned', (s) => s.state.mode === 'pinned');
if (suppressed.visible) throw Error('Pinned mode bypassed emergency latch');
await command('unlock', (s) => !s.state.positionLocked);
const editing = await command('edit', (s) => s.state.editingLayout && !s.native.clickthrough);
if (editing.native.clickthrough) throw Error('Edit mode still clickthrough');
await writeFile(join(directory, 'command.tmp'), 'capture');
await rename(join(directory, 'command.tmp'), join(directory, 'command.txt'));
await delay(500);
await command('finish', (s) => !s.state.editingLayout && s.native.clickthrough);
await command('hold', (s) => !s.visible && s.state.mode === 'hold');
await command('lock', (s) => s.state.positionLocked);
const report = {
  date: new Date().toISOString(),
  cycles: samples.length,
  initial,
  samples,
  result:
    'PASS: native visibility, nonactivation, window properties, emergency priority, edit transition',
  limitations: [
    'No physical key input or mouse click was injected',
    'No screen-pixel end-to-end latency measurement',
    'Window transparency property and own-webview snapshot are not a desktop compositing test',
  ],
};
await writeFile(join(directory, 'native-smoke.json'), JSON.stringify(report, null, 2));
console.log(report.result, `(${samples.length} cycles)`);
