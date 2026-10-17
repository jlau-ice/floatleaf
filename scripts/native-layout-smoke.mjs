// Run with the native fixture's --hotkey mode occupying Control+Alt+F12.
import { readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const dir = resolve(process.argv[2] || '.local/p2');
async function status() {
  return JSON.parse(await readFile(join(dir, 'status.json'), 'utf8'));
}
async function command(value, predicate) {
  await writeFile(join(dir, 'command.txt'), value);
  for (let i = 0; i < 120; i++) {
    await delay(30);
    const s = await status();
    if (predicate(s)) return s;
  }
  throw Error('Native action timeout: ' + value);
}
async function evaluate(body, overlay = false) {
  await rm(join(dir, 'ui-result.json'), { force: true });
  const script = `(async()=>{
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const delay = ms => new Promise(r => setTimeout(r, ms));
    const state = () => invoke('reader_state');
    const assert = (value, message) => { if(!value) throw Error(message); };
    try { ${body} }
    catch(e) { await invoke('reader_action', {action:'probe-result:'+JSON.stringify({ok:false,error:String(e)})}); }
  })();`;
  await writeFile(join(dir, overlay ? 'evaluate-overlay.js' : 'evaluate.js'), script);
  await writeFile(join(dir, 'command.txt'), overlay ? 'evaluate-overlay' : 'evaluate');
  for (let i = 0; i < 150; i++) {
    await delay(50);
    let report;
    try {
      report = JSON.parse(await readFile(join(dir, 'ui-result.json'), 'utf8'));
    } catch {
      continue;
    }
    if (!report.ok) throw Error(report.error);
    return report;
  }
  throw Error('WebView evaluation timed out');
}
const shortcut = await evaluate(`
  let initial = await state();
  await invoke('save_settings', {settings:{...initial.settings, shortcuts:{...initial.settings.shortcuts, display:'Control+Alt+Space'}, positionLocked:false}}); await delay(200);
  await invoke('reader_action', {action:'edit'}); await delay(100);
  await invoke('reader_action', {action:'finish'}); await delay(100);
  initial = await state();
  await invoke('save_settings', {settings:{...initial.settings, mode:'pinned'}, selectMode:true});
  await delay(200); assert((await state()).visible, 'Explicit pinned selection must show');
  let error = '';
  try { await invoke('save_settings', {settings:{...initial.settings, shortcuts:{...initial.settings.shortcuts, display:'Control+Alt+F12'}}}); }
  catch(e) { error = String(e); }
  assert(error.includes('注册失败'), 'Expected native hotkey conflict: '+error);
  assert((await state()).settings.shortcuts.display === initial.settings.shortcuts.display, 'Conflict changed old keys');
  const saved = {...initial.settings, shortcuts:{...initial.settings.shortcuts, display:'Control+Alt+KeyR'}, positionLocked:false};
  await invoke('save_settings', {settings:saved}); await delay(200);
  const changed = await state();
  assert(changed.settings.shortcuts.display === saved.shortcuts.display && !changed.visible && changed.emergencyLatched, 'Successful rebind must hide');
  const library = await invoke('library');
  assert(library.settings.shortcuts.display === saved.shortcuts.display, 'Keys not saved in SQLite');
  await invoke('save_settings', {settings:{...saved, shortcuts:initial.settings.shortcuts}}); await delay(150);
  await invoke('reader_action', {action:'pinned'}); await delay(100);
  assert(!(await state()).visible, 'Mode switch bypassed emergency');
  await invoke('reader_action', {action:'edit'}); await delay(150);
  await invoke('reader_action', {action:'probe-result:'+JSON.stringify({ok:true,error,settings:(await state()).settings})});
`);
const resize = await evaluate(
  `
  const initial = await state();
  await invoke('resize_overlay', {width:780, lines:4, persist:true}); await delay(400);
  const resized = await state();
  assert(resized.settings.width === 780 && resized.settings.lines === 4, 'Resize not applied');
  assert(resized.reader.anchor === initial.reader.anchor && resized.reader.ready, 'Resize lost reading position or layout');
  await invoke('reader_action', {action:'probe-result:'+JSON.stringify({ok:true,anchor:resized.reader.anchor})});
`,
  true,
);
const nativeSize = await status();
if (nativeSize.size.width !== 1560 || nativeSize.size.height !== 312)
  throw Error('Unexpected 2x physical window size');
await command('move-test', (s) => s.position.x === 240 && s.position.y === 220);
await delay(1200);
const moved = await status();
await command('finish', (s) => !s.state.editingLayout && s.native.clickthrough);
const recovered = await command('reset-position', (s) => s.position.x !== 240);
const area = recovered.monitors[0].workArea;
if (
  recovered.position.x < area.position.x ||
  recovered.position.y < area.position.y ||
  recovered.position.x + recovered.size.width > area.position.x + area.size.width ||
  recovered.position.y + recovered.size.height > area.position.y + area.size.height
)
  throw Error('Recovered window outside work area');
await command('move-test', (s) => s.position.x === 240 && s.position.y === 220);
await command('hold', (s) => !s.visible);
await delay(1200);
const report = {
  date: new Date().toISOString(),
  ok: true,
  shortcut,
  resize,
  nativeSize,
  moved,
  recovered,
  final: await status(),
  limitations: [
    'IPC and native APIs, no physical drag/keys',
    'One monitor at scale 2; actual disconnect/DPI pending',
  ],
};
await writeFile(join(dir, 'layout-result.json'), JSON.stringify(report, null, 2));
console.log(
  'PASS native shortcut conflict/rebind, complete-row resize, anchor, placement and reset',
);
