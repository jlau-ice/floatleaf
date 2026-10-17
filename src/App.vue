<script setup lang="ts">
import { ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';

interface RuntimeInfo { version: string; platform: string; architecture: string }
const desktop = isTauri();
const message = ref(desktop ? '桌面已启动，可以检查原生连接。' : '当前是网页预览。请用 pnpm desktop:dev 启动桌面程序。');
const checking = ref(false);

async function checkConnection() {
  if (!desktop || checking.value) return;
  checking.value = true;
  try {
    const info = await invoke<RuntimeInfo>('runtime_info');
    message.value = `原生连接正常 · v${info.version} · ${info.platform} ${info.architecture}`;
  } catch (error) {
    message.value = `连接检查失败：${String(error)}`;
  } finally {
    checking.value = false;
  }
}
</script>

<template>
  <main class="workspace">
    <header><span class="wordmark">FLOATLEAF</span><span class="edition">开发起点 / 0.1</span></header>
    <section class="intro" aria-labelledby="title">
      <p class="eyebrow">桌面悬浮阅读工具</p>
      <h1 id="title">浮页<span aria-hidden="true">。</span></h1>
      <p class="description">让文字轻轻浮在桌面上。<br />按住阅读，松手隐藏。</p>
    </section>
    <section class="next" aria-label="开发状态">
      <p class="number">01</p>
      <div><h2>下一步，验证悬浮阅读</h2><p>当前为初始化骨架。透明窗口、快捷键和小说导入尚待实现。</p><p class="hint">从 TASKS.md 的 P0-01 开始。</p></div>
    </section>
    <footer>
      <button type="button" :disabled="!desktop || checking" @click="checkConnection">{{ checking ? '检查中…' : '检查桌面连接' }}<span aria-hidden="true"> ↗</span></button>
      <p role="status" aria-live="polite">{{ message }}</p>
    </footer>
  </main>
</template>

<style>
:root { font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif; color: #203b32; background: #f4f2e9; font-synthesis: none; -webkit-font-smoothing: antialiased; }
* { box-sizing: border-box; }
body { margin: 0; }
.workspace { max-width: 1040px; margin: auto; padding: 36px 56px; }
header { display: flex; justify-content: space-between; gap: 16px; align-items: center; font-size: 12px; }
.wordmark { letter-spacing: .2em; font-weight: 650; }
.edition { color: #596a5e; }
.intro { padding: 54px 0 36px; }
.eyebrow { font-size: 12px; letter-spacing: .16em; color: #596a5e; }
h1 { font-family: 'Songti SC', 'STSong', 'SimSun', serif; font-size: clamp(64px, 10vw, 100px); font-weight: 500; line-height: 1.1; margin: 16px 0; letter-spacing: -.04em; }
h1 span { color: #ab7247; }
.description { font-size: 18px; line-height: 1.9; margin: 0; }
.next { display: flex; gap: 28px; border-top: 1px solid #c8cec0; padding: 22px 0; }
.number { font-family: Georgia, serif; font-size: 28px; color: #8c684c; margin: 0; }
h2 { font-size: 17px; font-weight: 550; margin: 0 0 8px; }
.next p:not(.number) { font-size: 13px; line-height: 1.8; margin: 0; color: #58665b; }
.next .hint { margin-top: 4px; }
footer { display: flex; align-items: center; gap: 18px; flex-wrap: wrap; padding-top: 8px; }
button { border: 1px solid #203b32; background: #203b32; color: #fffdf4; border-radius: 4px; padding: 11px 16px; font: inherit; font-size: 13px; cursor: pointer; }
button:hover:enabled { background: #315343; }
button:focus-visible { outline: 3px solid #ad774f; outline-offset: 4px; }
button:disabled { background: #e5e7dd; color: #58665b; border-color: #bcc4b6; cursor: default; }
footer p { max-width: 470px; font-size: 12px; line-height: 1.7; margin: 0; color: #58665b; overflow-wrap: anywhere; }
@media (max-width: 640px) { .workspace { padding: 26px; } .intro { padding-top: 40px; } .next { gap: 18px; } }
</style>
