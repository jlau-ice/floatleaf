<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useReader } from '../lib/desktop';
import { Paginator, textStyle } from '../lib/pagination';
const { state, action, error } = useReader();
const pageText = ref('');
let paginator: Paginator | undefined;
let loadedBook = '';
let content = '';
let generation = 0;
let layoutKey = '';
const styles = computed(() =>
  state.value
    ? {
        ...textStyle(state.value.settings),
        color: state.value.settings.color,
        opacity: state.value.settings.textOpacity,
      }
    : {},
);
const background = computed(
  () => `rgba(15, 25, 20, ${state.value?.settings.backgroundOpacity ?? 0})`,
);
watch(
  [
    () => state.value?.reader.bookId,
    () => state.value?.reader.anchor,
    () => state.value?.reader.layoutVersion,
  ],
  async () => {
    const snapshot = state.value;
    if (!snapshot) return;
    const request = ++generation;
    const reader = snapshot.reader;
    if (!reader.bookId) {
      pageText.value = '';
      paginator?.destroy();
      paginator = undefined;
      loadedBook = '';
      return;
    }
    try {
      if (loadedBook !== reader.bookId) {
        const text = await invoke<string>('book_text', { id: reader.bookId });
        if (request !== generation) return;
        content = text;
        loadedBook = reader.bookId;
        paginator?.destroy();
        paginator = undefined;
      }
      await document.fonts.ready;
      if (request !== generation) return;
      const key = JSON.stringify(snapshot.settings);
      if (!paginator || key !== layoutKey) {
        paginator?.destroy();
        paginator = new Paginator(content, snapshot.settings);
        layoutKey = key;
      }
      const page = paginator.page(reader.anchor);
      const previous = paginator.previous(reader.anchor);
      pageText.value = page.text;
      await invoke('report_layout', {
        report: {
          bookId: reader.bookId,
          anchor: reader.anchor,
          version: reader.layoutVersion,
          end: page.end,
          previous,
        },
      });
      error.value = '';
    } catch (e) {
      if (request === generation) error.value = String(e);
    }
  },
);
onUnmounted(() => {
  generation++;
  paginator?.destroy();
});
async function drag(event: MouseEvent) {
  if (event.button === 0) {
    try {
      await invoke('drag_overlay');
    } catch (e) {
      error.value = String(e);
    }
  }
}
</script>
<template>
  <section
    class="overlay"
    :class="{ editing: state?.editingLayout }"
    :style="{ backgroundColor: background }"
  >
    <div v-if="state?.editingLayout" class="layout-toolbar">
      <button class="drag" @mousedown="drag">⠿ 拖动阅读区域</button>
      <button @click="action('finish')">完成布局</button>
    </div>
    <div v-if="pageText" class="reading-text" :style="styles">{{ pageText }}</div>
    <div v-else class="empty-text">
      {{ state?.reader.bookId ? '已到末尾，可返回上一页。' : '请在浮页设置中导入 TXT 小说。' }}
    </div>
    <span v-if="error" class="overlay-error">{{ error }}</span>
  </section>
</template>
<style scoped>
.overlay {
  width: 100vw;
  height: 100vh;
  padding: 32px 20px 16px;
  overflow: hidden;
}
.reading-text {
  text-shadow:
    0 1px 3px #000,
    0 0 1px #000;
}
.empty-text {
  font-size: 14px;
  line-height: 24px;
  color: #fff;
  text-shadow: 0 1px 3px #000;
}
.editing {
  outline: 1px dashed #6a9b82;
  outline-offset: -1px;
}
.layout-toolbar {
  position: absolute;
  top: 3px;
  left: 8px;
  right: 8px;
  display: flex;
  justify-content: space-between;
}
.layout-toolbar button {
  font-size: 11px;
  padding: 4px 8px;
  background: #f4f2e9;
  color: #203b32;
}
.drag {
  cursor: move;
}
.overlay-error {
  position: absolute;
  bottom: 0;
  font-size: 10px;
  color: #fff;
  background: #873b25;
}
</style>
