import { mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const dir = resolve(process.argv[2] || '.local/p3-smoke');
await mkdir(dir, { recursive: true });
const first = join(dir, 'final-first.txt'),
  second = join(dir, 'final-second.txt');
await writeFile(first, '合成阅读回归文本。Hello world.\n'.repeat(300));
await writeFile(second, '另一本合成测试文本。\n'.repeat(100));
await rm(join(dir, 'ui-result.json'), { force: true });
await writeFile(
  join(dir, 'evaluate.js'),
  `(async()=>{
 const invoke=window.__TAURI_INTERNALS__.invoke;
 const delay=ms=>new Promise(r=>setTimeout(r,ms));
 const state=()=>invoke('reader_state');
 const assert=(value,message)=>{if(!value)throw Error(message);};
 async function wait(f){for(let n=0;n<150;n++){let s=await state();if(f(s))return s;await delay(30);}throw Error('state timeout');}
 try {
  const a=await invoke('import_book',{path:${JSON.stringify(first)},encoding:'utf8'});
  await wait(s=>s.reader.bookId===a.id&&s.reader.ready);
  const b=await invoke('import_book',{path:${JSON.stringify(second)},encoding:'utf8'});
  await wait(s=>s.reader.bookId===b.id&&s.reader.ready);
  await invoke('select_book',{id:a.id});await wait(s=>s.reader.bookId===a.id&&s.reader.ready);
  const initial=await state();
  await invoke('save_settings',{settings:{...initial.settings,mode:'pinned',intervalSeconds:2},selectMode:true});await wait(s=>s.visible&&s.reader.ready);
  await invoke('reader_action',{action:'auto'});await wait(s=>s.reader.autoRunning);
  await delay(500); const beforeStyle = await state();
  await invoke('save_settings',{settings:{...beforeStyle.settings,backgroundOpacity:0.2}}); await delay(100);
  const styled = await state();
  assert(styled.reader.layoutVersion === beforeStyle.reader.layoutVersion, 'Color change rebuilt pagination');
  assert(styled.reader.remainingMs <= beforeStyle.reader.remainingMs, 'Appearance restarted countdown');
  await invoke('remove_book',{id:b.id});await delay(100);
  const kept=await state();assert(kept.reader.autoRunning&&kept.reader.bookId===a.id,'Removing another book interrupted reading');
  await invoke('remove_book',{id:a.id});await wait(s=>!s.visible&&s.reader.bookId===null);
  document.querySelectorAll('.tabs button')[2].click();await delay(100);
  const input=document.querySelector('.shortcut-field input');
  const tab=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true});input.dispatchEvent(tab);
  assert(!tab.defaultPrevented,'Shortcut recorder trapped Tab navigation');
  input.dispatchEvent(new KeyboardEvent('keydown',{key:'r',code:'KeyR',ctrlKey:true,altKey:true,bubbles:true,cancelable:true}));
  [...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='保存快捷键').click();
  await wait(s=>s.settings.shortcuts.display==='Control+Alt+KeyR');
  const shelf=await invoke('library');assert(shelf.settings.shortcuts.display==='Control+Alt+KeyR','UI save not persisted');
  await invoke('save_settings',{settings:initial.settings});
  await wait(s=>s.settings.shortcuts.display===initial.settings.shortcuts.display);
  await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:true,checks:['inactive removal preserves running session','active removal clears reader and hides','shortcut UI records and saves actual bindings','Tab remains navigable'],state:await state()})});
 }catch(e){await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:false,error:String(e),state:await state()})});}
})();`,
);
await writeFile(join(dir, 'command.txt'), 'evaluate');
let completed = false;
for (let n = 0; n < 180; n++) {
  await delay(200);
  let result;
  try {
    result = JSON.parse(await readFile(join(dir, 'ui-result.json'), 'utf8'));
  } catch {
    continue;
  }
  completed = true;
  console.log(JSON.stringify(result, null, 2));
  if (!result.ok) process.exitCode = 1;
  break;
}

if (!completed) throw Error('Final native smoke timed out');
