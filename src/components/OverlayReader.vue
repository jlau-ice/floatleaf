<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { useReader } from '../lib/desktop';
const { state, action, error } = useReader();
async function drag(event: MouseEvent) {
  if (event.button === 0) {
    try { await invoke('drag_overlay'); } catch (e) { error.value = String(e); }
  }
}
</script>
<template>
  <section class="overlay" :class="{ editing: state?.editingLayout }">
    <div v-if="state?.editingLayout" class="layout-toolbar">
      <button class="drag" @mousedown="drag">⠿ 拖动阅读区域</button>
      <button @click="action('finish')">完成布局</button>
    </div>
    <div class="reading-text">风吹过书页，文字轻轻浮在桌面上。<br>按住阅读，松手隐藏。这是浮页的窗口验证文本。</div>
    <span v-if="error" class="overlay-error">{{ error }}</span>
  </section>
</template>
<style scoped>
.overlay { width: 100vw; height: 100vh; padding: 36px 20px 12px; overflow: hidden; color: rgba(255,255,255,.85); }
.reading-text { font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif; font-size: 18px; line-height: 27px; text-shadow: 0 1px 3px #000, 0 0 1px #000; white-space: pre-wrap; }
.editing { outline: 1px dashed #6a9b82; outline-offset: -1px; background: #203b32a6; }
.layout-toolbar { position: absolute; top: 3px; left: 8px; right: 8px; display: flex; justify-content: space-between; }
.layout-toolbar button { font-size: 11px; padding: 4px 8px; background: #f4f2e9; color: #203b32; }
.drag { cursor: move; }
.overlay-error { font-size: 10px; color: #fff; }
</style>
