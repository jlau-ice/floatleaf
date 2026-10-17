// Actual WebView -> IPC -> reader -> SQLite regression; requires a fresh test data directory.
import { mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const dir = resolve(process.argv[2] || '.local/auto-smoke');
await mkdir(dir, { recursive: true });
const fixture = join(dir, 'auto-fixture.txt');
await writeFile(fixture, '自动翻页合成验收文本。Hello FloatLeaf.\n'.repeat(200));
await rm(join(dir, 'ui-result.json'), { force: true });
await writeFile(
  join(dir, 'evaluate.js'),
  `(async()=>{
 const invoke=window.__TAURI_INTERNALS__.invoke;
 const delay=ms=>new Promise(r=>setTimeout(r,ms));
 const assert=(value,message)=>{if(!value)throw Error(message);};
 const state=()=>invoke('reader_state');
 async function wait(predicate){for(let n=0;n<150;n++){const s=await state();if(predicate(s))return s;await delay(30);}throw Error('state timeout');}
 const autoButton=()=>[...document.querySelectorAll('button')].find(b=>['自动翻页','暂停自动翻页'].includes(b.textContent.trim()));
 try {
  const book=await invoke('import_txt',{path:${JSON.stringify(fixture)},encoding:'utf8'});
  await wait(s=>s.reader.bookId===book.id&&s.reader.ready);
  await invoke('reader_action',{action:'hold'});
  await wait(s=>s.mode==='hold'&&!s.visible);
  await delay(100);
  const slider=document.querySelector('[aria-label="自动翻页间隔（秒）"]');
  assert(slider,'Interval control is missing from the reading controls');
  slider.value='3';slider.dispatchEvent(new Event('input',{bubbles:true}));slider.dispatchEvent(new Event('change',{bubbles:true}));
  await wait(s=>s.settings.intervalSeconds===3&&s.reader.intervalMs===3000);
  assert((await invoke('library')).settings.intervalSeconds===3,'Interval was not stored in SQLite');
  assert(!autoButton().disabled,'Hold mode disabled the hidden auto button');
  autoButton().click();await wait(s=>s.reader.autoRunning&&!s.visible);
  await delay(3400);
  let s=await state();
  assert(s.reader.anchor===0&&s.reader.remainingMs===3000,'Hidden reader advanced or consumed countdown');
  assert(document.querySelector('.timer').textContent.includes('显示后继续'),'Hidden timer hint is missing');
  autoButton().click();await wait(s=>!s.reader.autoRunning&&!s.visible);
  autoButton().click();await wait(s=>s.reader.autoRunning&&!s.visible);
  await invoke('reader_action',{action:'pinned'});
  await wait(s=>s.visible&&s.reader.autoRunning);
  await wait(s=>s.reader.anchor>0&&s.reader.ready);
  await invoke('reader_action',{action:'hold'});
  const hidden=await wait(s=>!s.visible&&s.reader.autoRunning);
  await delay(350);
  s=await state();assert(s.reader.anchor===hidden.reader.anchor&&s.reader.remainingMs===hidden.reader.remainingMs,'Hiding did not pause countdown');
  autoButton().click();await wait(s=>!s.reader.autoRunning&&!s.visible);
  await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:true,checks:['hidden hold mode UI can enable and pause auto','interval UI updates reader and SQLite','hidden countdown stays frozen','visible reader advances at configured interval','hiding pauses remaining time'],state:await state()})});
 }catch(e){await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:false,error:String(e),state:await state()})});}
})();`,
);
await writeFile(join(dir, 'command.txt'), 'evaluate');
let result;
for (let n = 0; n < 180; n++) {
  await delay(200);
  try {
    result = JSON.parse(await readFile(join(dir, 'ui-result.json'), 'utf8'));
    break;
  } catch {
    continue;
  }
}
if (!result) throw Error('Auto reading native regression timed out');
console.log(JSON.stringify(result, null, 2));
if (!result.ok) process.exitCode = 1;
