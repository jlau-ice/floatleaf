<script setup lang="ts">
import { desktop, useReader } from './lib/desktop';
import OverlayReader from './components/OverlayReader.vue';
const overlay = new URLSearchParams(location.search).get('window') === 'overlay';
document.documentElement.classList.toggle('overlay-root', overlay);
const { state, error, action } = useReader();
</script>

<template>
  <OverlayReader v-if="overlay" />
  <main v-else class="workspace">
    <header><span class="wordmark">FLOATLEAF</span><span class="edition">浮页 / 原生窗口验证</span></header>
    <section class="intro"><p class="eyebrow">留一点空间，给文字。</p><h1>阅读，在一旁<span>。</span></h1>
      <p class="description">按住 <kbd>Ctrl</kbd> + <kbd>Alt / ⌥</kbd> + <kbd>空格</kbd>，文字浮现。<br>松开任一组成键，回到手边的事。</p>
    </section>
    <p v-if="!desktop" class="notice">网页预览无法控制系统窗口。请使用 pnpm desktop:dev 启动桌面程序。</p>
    <p v-if="error || state?.error" class="notice error" role="alert">{{ error || state?.error }}</p>
    <section class="controls" :aria-disabled="!desktop">
      <div class="row"><div><h2>显示方式</h2><p>隐藏后重新按下快捷键即可恢复。</p></div>
        <div class="segmented" aria-label="显示方式"><button v-for="item in [['hold','按住'],['toggle','切换'],['pinned','固定']]" :key="item[0]" :disabled="!desktop" :aria-pressed="state?.mode === item[0]" @click="action(item[0]!)">{{ item[1] }}</button></div>
      </div>
      <div class="row"><div><h2>阅读区域</h2><p>阅读时鼠标穿透。解除锁定后可编辑布局。</p></div>
        <div class="actions"><button :disabled="!desktop" class="secondary" @click="action(state?.positionLocked ? 'unlock' : 'lock')">{{ state?.positionLocked ? '解除位置锁定' : '锁定位置' }}</button><button :disabled="!desktop || state?.positionLocked" @click="action(state?.editingLayout ? 'finish' : 'edit')">{{ state?.editingLayout ? '完成布局' : '编辑布局' }}</button></div>
      </div>
      <div class="row"><div><h2>紧急隐藏</h2><p><kbd>Ctrl</kbd> + <kbd>Alt / ⌥</kbd> + <kbd>Esc</kbd> · 优先于固定显示</p></div><button class="secondary" :disabled="!desktop" @click="action('hide')">立即隐藏</button></div>
    </section>
    <footer><span class="status-dot" :class="{ visible: state?.visible }"></span><span role="status">{{ state?.emergencyLatched ? '已紧急隐藏 · 完全松开旧快捷键后重新触发' : state?.visible ? '阅读窗口已显示' : '阅读窗口已隐藏' }}</span><span class="footnote">关闭设置后，可从托盘或菜单栏找回。</span></footer>
  </main>
</template>

<style>
:root { font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif; color: #203b32; background: #f4f2e9; font-synthesis: none; -webkit-font-smoothing: antialiased; }
* { box-sizing: border-box; }
body { margin: 0; }
.overlay-root, .overlay-root body, .overlay-root #app { background: transparent !important; overflow: hidden; }
.workspace { max-width: 1040px; margin: auto; padding: 30px 44px; }
header { display: flex; justify-content: space-between; gap: 16px; align-items: center; font-size: 11px; }
.wordmark { letter-spacing: .2em; font-weight: 650; }
.edition { color: #596a5e; }
.intro { padding: 32px 0 28px; }
.eyebrow { font-size: 12px; letter-spacing: .16em; color: #596a5e; }
h1 { font-family: 'Songti SC', 'STSong', 'SimSun', serif; font-size: clamp(38px, 6vw, 58px); font-weight: 500; line-height: 1.2; margin: 16px 0; letter-spacing: -.04em; }
h1 span { color: #ab7247; }
.description { font-size: 14px; line-height: 2; margin: 0; }
kbd { font-family: inherit; font-size: .9em; border: 1px solid #c5cabd; border-radius: 4px; padding: 2px 5px; }
.row { display: flex; justify-content: space-between; align-items: center; gap: 20px; border-top: 1px solid #c8cec0; padding: 19px 0; }
h2 { font-size: 15px; font-weight: 550; margin: 0 0 7px; }
.row p { font-size: 12px; line-height: 1.8; margin: 0; color: #58665b; }
button { border: 1px solid #203b32; background: #203b32; color: #fffdf4; border-radius: 4px; padding: 9px 12px; font: inherit; font-size: 12px; cursor: pointer; white-space: nowrap; }
button:hover:enabled { background: #315343; color: #fffdf4; }
button:focus-visible { outline: 3px solid #ad774f; outline-offset: 3px; }
button:disabled { opacity: .45; cursor: default; }
.secondary { background: transparent; color: #203b32; border-color: #a7b4a5; }
.actions, .segmented { display: flex; gap: 8px; }
.segmented { border: 1px solid #a7b4a5; border-radius: 5px; gap: 0; padding: 3px; }
.segmented button { background: transparent; color: #203b32; border-color: transparent; }
.segmented button[aria-pressed=true] { background: #203b32; color: #fffdf4; }
footer { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; padding-top: 14px; font-size: 11px; color: #58665b; }
.footnote { margin-left: auto; }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #839080; }
.status-dot.visible { background: #328452; }
.notice { border-left: 3px solid #ab7247; padding: 10px; font-size: 12px; line-height: 1.6; background: #ede6d8; }
.error { color: #873b25; }
@media (max-width: 700px) { .workspace { padding: 24px; } .row { align-items: flex-start; flex-wrap: wrap; } .footnote { margin-left: 0; width: 100%; } }
</style>
