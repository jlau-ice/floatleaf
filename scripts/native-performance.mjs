// Optimized diagnostics build; measures main + own WebContent processes, not shared GPU/network services.
import { readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const dir = resolve(process.argv[2] || '.local/performance');
const binary = resolve(process.argv[3] || '.local/floatleaf-diagnostics');
await mkdir(dir, { recursive: true });
const child = spawn(binary, [], {
  env: { ...process.env, FLOATLEAF_PROBE_DIR: dir, FLOATLEAF_DATA_DIR: join(dir, 'data') },
  stdio: ['ignore', 'inherit', 'inherit'],
});
const status = async () => JSON.parse(await readFile(join(dir, 'status.json'), 'utf8'));
async function wait(predicate) {
  for (let i = 0; i < 200; i++) {
    await delay(50);
    try {
      const s = await status();
      if (predicate(s)) return s;
    } catch {}
  }
  throw Error('Native state timeout');
}
await wait((s) => s.pid === child.pid && s.native.webPid > 0 && s.mainNative.webPid > 0);
const file = join(dir, 'synthetic-performance.txt');
await writeFile(
  file,
  '合成性能文本：山风轻拂书页，阅读让日常慢下来。Hello FloatLeaf 👩‍💻 e\u0301。\n'.repeat(50000),
);
await rm(join(dir, 'ui-result.json'), { force: true });
await writeFile(
  join(dir, 'evaluate.js'),
  `(async()=>{
 const i=window.__TAURI_INTERNALS__.invoke;
 try {
  const s=await i('reader_state');
  await i('save_settings',{settings:{...s.settings,mode:'hold',width:600,fontSize:18,lineHeight:1.5,lines:2,intervalSeconds:2}});
  await i('import_book',{path:${JSON.stringify(file)},encoding:'utf8'});
  await i('reader_action',{action:'probe-result:'+JSON.stringify({ok:true})});
 }catch(e){await i('reader_action',{action:'probe-result:'+JSON.stringify({ok:false,error:String(e)})});}
})();`,
);
await writeFile(join(dir, 'command.txt'), 'evaluate');
await wait((s) => s.state.reader.ready && s.state.reader.total > 1000000);
await writeFile(join(dir, 'command.txt'), 'hide-settings');
await delay(5000);
function seconds(time) {
  return time
    .split(':')
    .reverse()
    .reduce((sum, v, index) => sum + Number(v) * 60 ** index, 0);
}
async function sample() {
  const s = await status();
  const pids = [...new Set([s.pid, s.native.webPid, s.mainNative.webPid])];
  const result = spawnSync('ps', ['-p', pids.join(','), '-o', 'pid=,time=,rss='], {
    encoding: 'utf8',
  });
  if (result.status !== 0) throw Error(result.stderr);
  const processes = result.stdout
    .trim()
    .split('\n')
    .map((line) => {
      const [pid, time, rss] = line.trim().split(/\s+/);
      return { pid: Number(pid), cpuSeconds: seconds(time), rssKiB: Number(rss) };
    });
  if (processes.length !== pids.length) throw Error('A measured WebView process exited');
  return {
    at: new Date().toISOString(),
    time: performance.now(),
    anchor: s.state.reader.anchor,
    remainingMs: s.state.reader.remainingMs,
    visible: s.visible,
    autoRunning: s.state.reader.autoRunning,
    processes,
    cpuSeconds: processes.reduce((sum, p) => sum + p.cpuSeconds, 0),
    rssMiB: processes.reduce((sum, p) => sum + p.rssKiB / 1024, 0),
  };
}
const phases = [];
for (const [name, duration, action] of [
  ['hidden-idle', 300, 'hold'],
  ['pinned-static', 60, 'pinned'],
  ['auto-reading', 600, 'auto'],
]) {
  await writeFile(join(dir, 'command.txt'), action);
  await delay(500);
  const samples = [await sample()];
  if (name === 'auto-reading' && !samples[0].autoRunning) throw Error('Auto reading did not start');
  console.log('Started', name, duration, 'seconds');
  const deadline = performance.now() + duration * 1000;
  while (performance.now() < deadline) {
    await delay(Math.min(5000, deadline - performance.now()));
    samples.push(await sample());
  }
  const expectedPids = samples[0].processes.map((p) => p.pid).join();
  if (samples.some((s) => s.processes.map((p) => p.pid).join() !== expectedPids))
    throw Error('A measured process restarted');
  if (samples.some((s) => s.visible !== (name !== 'hidden-idle')))
    throw Error('Visibility changed during measurement');
  if (name === 'auto-reading' && samples.some((s) => !s.autoRunning))
    throw Error('Auto reading was interrupted');
  const first = samples[0],
    last = samples.at(-1);
  if (name === 'hidden-idle' && (last.anchor !== first.anchor || last.visible))
    throw Error('Hidden state changed');
  if (name === 'auto-reading' && (last.anchor <= first.anchor || !last.autoRunning))
    throw Error('Auto reading stopped unexpectedly');
  const phase = {
    name,
    durationSeconds: (last.time - first.time) / 1000,
    cpuPercent: ((last.cpuSeconds - first.cpuSeconds) / ((last.time - first.time) / 1000)) * 100,
    rssStartMiB: first.rssMiB,
    rssEndMiB: last.rssMiB,
    rssMaxMiB: Math.max(...samples.map((s) => s.rssMiB)),
    samples,
  };
  phases.push(phase);
  await writeFile(
    join(dir, 'performance.json'),
    JSON.stringify(
      {
        date: new Date().toISOString(),
        build: 'optimized diagnostics',
        phases,
        limitations: [
          'file probe enabled',
          'main and own WebContent only; excludes shared GPU/network services',
          'pinned-static is not physical hold',
          'no end-to-end screen-pixel latency measurement',
        ],
      },
      null,
      2,
    ),
  );
  console.log(
    'Completed',
    name,
    JSON.stringify({
      cpuPercent: phase.cpuPercent,
      rssStartMiB: phase.rssStartMiB,
      rssEndMiB: phase.rssEndMiB,
      rssMaxMiB: phase.rssMaxMiB,
    }),
  );
}
await writeFile(join(dir, 'command.txt'), 'exit');
console.log('Performance measurement complete:', join(dir, 'performance.json'));
