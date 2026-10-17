// Runs in the real desktop WebView through the opt-in debug probe. Synthetic books only.
import { mkdir, writeFile, readFile, rename, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const dir = resolve(process.argv[2] || '.local/p1');
await mkdir(dir, { recursive: true });
const first = join(dir, 'fixture-utf8.txt'),
  second = join(dir, 'fixture-gbk.txt'),
  empty = join(dir, 'fixture-empty.txt');
await writeFile(
  first,
  '合成验收文本，不是真实小说。\n春风吹过窗台，Hello world 👩‍💻 e\u0301。\n\n'.repeat(90),
);
await writeFile(
  second,
  Buffer.concat(
    Array.from({ length: 100 }, () =>
      Buffer.from([0xd6, 0xd0, 0xce, 0xc4, 0x20, 0x47, 0x42, 0x4b, 0x0a]),
    ),
  ),
);
await writeFile(empty, '');
await rm(join(dir, 'ui-result.json'), { force: true });
const script = `
(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke;
  const delay = ms => new Promise(r => setTimeout(r, ms));
  const state = () => invoke('reader_state');
  const assert = (value, message) => { if (!value) throw Error(message); };
  const evidence = [];
  async function wait(predicate) {
    for (let i = 0; i < 160; i++) { const s = await state(); if (predicate(s)) return s; await delay(30); }
    throw Error('Timed out waiting for native reader state: ' + JSON.stringify(await state()));
  }
  async function action(action) { await invoke('reader_action', { action }); await delay(40); }
  async function turn(direction) {
    const before = await state(); await action(direction);
    return wait(s => s.reader.ready && s.reader.anchor !== before.reader.anchor);
  }
  async function settings(patch) {
    const old = await state(); await invoke('save_settings', { settings: { ...old.settings, ...patch } });
    return wait(s => s.reader.ready && s.reader.layoutVersion > old.reader.layoutVersion);
  }
  try {
    let rejected = false;
    try { await invoke('inspect_txt', { path: ${JSON.stringify(empty)}, encoding: 'auto' }); } catch { rejected = true; }
    assert(rejected, 'Empty import must be rejected');
    const gbk = await invoke('inspect_txt', { path: ${JSON.stringify(second)}, encoding: 'auto' });
    assert(gbk.encoding === 'GBK' && gbk.preview.startsWith('中文') && gbk.warning, 'GBK preview/fallback warning');
    const a = await invoke('import_txt', { path: ${JSON.stringify(first)}, encoding: 'auto' });
    await wait(s => s.reader.bookId === a.id && s.reader.ready);
    await settings({ mode: 'pinned', width: 600, lines: 2, fontSize: 18, intervalSeconds: 2 });
    await action('pinned');
    const stable = await state(); await delay(400);
    assert((await state()).revision === stable.revision, 'Idle layout is emitting repeated state events');
    await turn('next'); await turn('next');
    const forward = await turn('next'); const previous = await turn('previous');
    assert(previous.reader.anchor < forward.reader.anchor, 'Previous page must move backwards');
    const anchor = previous.reader.anchor;
    const reflow = await settings({ width: 400, fontSize: 24, lines: 3 });
    assert(reflow.reader.anchor === anchor, 'Reflow moved anchor');
    evidence.push('real WebView pagination and reflow preserved UTF-16 anchor');
    await action('auto'); await delay(500); await action('hold');
    const hidden = await state(); await delay(2300); const still = await state();
    assert(still.reader.anchor === hidden.reader.anchor && still.reader.remainingMs === hidden.reader.remainingMs, 'Hidden timer advanced');
    await action('pinned'); await wait(s => s.reader.ready && s.reader.anchor !== hidden.reader.anchor);
    await turn('next'); const manual = await state();
    assert(manual.reader.remainingMs > 1600, 'Manual turn did not reset timer');
    await action('auto');
    const savedA = (await state()).reader.anchor;
    const duplicate = await invoke('import_txt', { path: ${JSON.stringify(first)}, encoding: 'auto' });
    assert(duplicate.id === a.id && duplicate.anchor === savedA, 'Duplicate import lost progress');
    const b = await invoke('import_txt', { path: ${JSON.stringify(second)}, encoding: 'gbk' });
    await wait(s => s.reader.bookId === b.id && s.reader.ready);
    const pageB = await turn('next');
    await invoke('select_book', { id: a.id }); const restored = await wait(s => s.reader.bookId === a.id && s.reader.ready);
    assert(restored.reader.anchor === savedA, 'Per-book progress was not restored');
    await action('hide'); await action('pinned'); assert(!(await state()).visible, 'Emergency was bypassed');
    await action('unlock'); await action('edit'); await action('finish');
    await settings({ width: 1600, fontSize: 12, lines: 12, intervalSeconds: 2 });
    for (let i = 0; i < 100; i++) { const s = await state(); if (s.reader.end === s.reader.total) break; await turn('next'); }
    const end = await state(); assert(end.reader.end === end.reader.total && !end.reader.autoRunning, 'End did not stop auto paging');
    await settings({ mode: 'hold', width: 600, fontSize: 18, lines: 2, positionLocked: true });
    await delay(1200);
    const shelf = await invoke('library');
    assert(shelf.books.length === 2, 'Duplicate book row');
    assert(shelf.books.find(v => v.id === b.id).anchor === pageB.reader.anchor, 'Second book progress lost');
    const final = await state();
    evidence.push('UTF-8/GBK import, duplicate detection, per-book progress, hide pause, resume, manual reset, emergency and end stop');
    await invoke('reader_action', { action: 'probe-result:' + JSON.stringify({ ok: true, evidence, bookA: a.id, bookB: b.id, finalAnchor: final.reader.anchor, finalSettings: final.settings, library: shelf }) });
  } catch (error) {
    await invoke('reader_action', { action: 'probe-result:' + JSON.stringify({ ok: false, error: String(error), state: await state(), evidence }) });
  }
})();`;
await writeFile(join(dir, 'evaluate.js'), script);
await writeFile(join(dir, 'command.tmp'), 'evaluate');
await rename(join(dir, 'command.tmp'), join(dir, 'command.txt'));
for (let i = 0; i < 180; i++) {
  await delay(500);
  try {
    const report = JSON.parse(await readFile(join(dir, 'ui-result.json'), 'utf8'));
    console.log(JSON.stringify(report, null, 2));
    if (!report.ok) process.exitCode = 1;
    break;
  } catch (error) {
    if (i === 179) throw error;
  }
}
