// Real WKWebView/IPC/SQLite flow; run with isolated debug app data and synthetic fixtures.
import { mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';

const directory = resolve(process.argv[2] || '.local/chapter-smoke');
await mkdir(directory, { recursive: true });
const fixtures = join(directory, 'fixtures');
execFileSync('python3', ['scripts/create-chapter-fixtures.py', fixtures]);
await rm(join(directory, 'ui-result.json'), { force: true });
await writeFile(
  join(directory, 'evaluate.js'),
  `(async()=>{
 const invoke=window.__TAURI_INTERNALS__.invoke;
 const delay=ms=>new Promise(r=>setTimeout(r,ms));
 const assert=(value,message)=>{if(!value)throw Error(message);};
 const state=()=>invoke('reader_state');
 async function wait(predicate){for(let n=0;n<200;n++){const s=await state();if(predicate(s))return s;await delay(30);}throw Error('state timeout');}
 const checks=[];
 try {
   const files=['chapters-utf8.txt','chapters-gbk.txt','chapters-epub2.epub','chapters-epub3.epub'];
   let book, chapters;
   for(const file of files){
     const path=${JSON.stringify(fixtures)}+'/'+file;
     const preview=await invoke('inspect_book',{path,encoding:'auto'});
     assert(preview.chapterCount===3,file+' chapter count');
     book=await invoke('import_book',{path,encoding:'auto'});
     await wait(s=>s.reader.bookId===book.id&&s.reader.ready);
     chapters=await invoke('book_chapters',{id:book.id});
     const content=await invoke('book_text',{id:book.id});
     assert(chapters.length===3,file+' stored chapters');
     assert(content.slice(chapters[1].anchor).startsWith('第二章 山路'),file+' UTF-16 anchor mismatch');
     checks.push(file+' imported with correct directory and UTF-16 anchors');
   }
   await invoke('reader_action',{action:'hold'});
   await wait(s=>!s.visible);
   await delay(150);
   const details=document.querySelector('.chapter-directory details');
   assert(details,'Directory UI missing');details.open=true;await delay(80);
   const search=document.querySelector('[aria-label="搜索章节"]');
   search.value='归来';search.dispatchEvent(new Event('input',{bubbles:true}));await delay(60);
   const buttons=document.querySelectorAll('.chapter-list button');
   assert(buttons.length===1&&!buttons[0].disabled,'Filtered directory is not selectable while hidden');
   buttons[0].click();
   await wait(s=>s.reader.ready&&s.reader.anchor===chapters[2].anchor&&!s.visible);
   assert(document.querySelector('.chapter-current strong').textContent==='第三章 归来','Current chapter label did not update');
   let s=await state();
   await invoke('save_settings',{settings:{...s.settings,fontSize:24,width:480,intervalSeconds:3}});
   await wait(s=>s.reader.ready&&s.settings.fontSize===24);
   assert((await state()).reader.anchor===chapters[2].anchor,'Reflow moved the chapter anchor');
   await invoke('reader_action',{action:'auto'});
   await wait(s=>s.reader.autoRunning);
   const previous=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='← 上一章');
   previous.click();
   s=await wait(s=>s.reader.ready&&s.reader.anchor===chapters[1].anchor);
   assert(!s.visible&&s.reader.autoRunning&&s.reader.remainingMs===3000,'Chapter jump changed visibility or auto intent');
   await delay(3200);s=await state();
   assert(s.reader.anchor===chapters[1].anchor&&s.reader.remainingMs===3000,'Hidden chapter jump advanced reading');
   await invoke('reader_action',{action:'auto'});await wait(s=>!s.reader.autoRunning);
   const saved=(await invoke('library')).books.find(b=>b.id===book.id);
   assert(saved.anchor===chapters[1].anchor,'Chapter selection was not saved');
   search.value='';search.dispatchEvent(new Event('input',{bubbles:true}));
   checks.push('real directory search + hidden selection + previous chapter + reflow + timer freeze + SQLite progress');
   await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:true,checks,bookId:book.id,anchor:chapters[1].anchor,chapters,state:await state()})});
 }catch(e){await invoke('reader_action',{action:'probe-result:'+JSON.stringify({ok:false,error:String(e),state:await state()})});}
})();`,
);
await writeFile(join(directory, 'command.txt'), 'evaluate');
let result;
for (let i = 0; i < 180; i++) {
  await delay(200);
  try {
    result = JSON.parse(await readFile(join(directory, 'ui-result.json'), 'utf8'));
    break;
  } catch {
    continue;
  }
}
if (!result) throw Error('Native chapter regression timed out');
console.log(JSON.stringify(result, null, 2));
if (!result.ok) process.exitCode = 1;
