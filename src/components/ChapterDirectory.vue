<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Chapter } from '../lib/desktop';

const props = defineProps<{ bookId: string; anchor: number; busy: boolean }>();
const emit = defineEmits<{
  select: [bookId: string, index: number];
  error: [message: string];
}>();
const chapters = ref<Chapter[]>([]);
const search = ref('');
const loading = ref(false);
const expanded = ref(false);
const list = ref<HTMLOListElement>();
watch(
  () => props.bookId,
  async (id) => {
    chapters.value = [];
    search.value = '';
    loading.value = true;
    try {
      const result = await invoke<Chapter[]>('book_chapters', { id });
      if (props.bookId === id) chapters.value = result;
    } catch (error) {
      if (props.bookId === id) emit('error', String(error));
    } finally {
      if (props.bookId === id) loading.value = false;
    }
  },
  { immediate: true },
);
const currentIndex = computed(() => {
  for (let i = chapters.value.length - 1; i >= 0; i--) {
    if (chapters.value[i].anchor <= props.anchor) return i;
  }
  return -1;
});
const currentTitle = computed(() => chapters.value[currentIndex.value]?.title);
const matches = computed(() => {
  const query = search.value.trim().toLocaleLowerCase();
  return chapters.value
    .map((chapter, index) => ({ ...chapter, index }))
    .filter((c) => c.title.toLocaleLowerCase().includes(query));
});
async function revealCurrent(event: Event) {
  expanded.value = (event.target as HTMLDetailsElement).open;
  if (expanded.value) {
    await nextTick();
    list.value?.querySelector('[aria-current="location"]')?.scrollIntoView({ block: 'nearest' });
  }
}
</script>

<template>
  <section class="chapter-directory" aria-label="章节导航">
    <div class="chapter-current">
      <span
        >当前章节
        <strong>{{ currentTitle || (loading ? '正在读取…' : '未识别到章节') }}</strong></span
      >
      <div class="chapter-actions">
        <button
          class="secondary compact"
          :disabled="busy || currentIndex <= 0"
          @click="emit('select', bookId, currentIndex - 1)"
        >
          ← 上一章
        </button>
        <button
          class="secondary compact"
          :disabled="busy || !chapters.length || currentIndex >= chapters.length - 1"
          @click="emit('select', bookId, currentIndex + 1)"
        >
          下一章 →
        </button>
      </div>
    </div>
    <details v-if="chapters.length" @toggle="revealCurrent">
      <summary>
        章节目录 <span>{{ chapters.length }} 节 · 选择章节跳转</span>
      </summary>
      <template v-if="expanded">
        <input
          v-model="search"
          class="chapter-search"
          type="search"
          placeholder="搜索章节标题"
          aria-label="搜索章节"
        />
        <ol ref="list" class="chapter-list" aria-label="章节列表">
          <li v-for="chapter in matches" :key="chapter.index">
            <button
              :disabled="busy"
              :aria-current="chapter.index === currentIndex ? 'location' : undefined"
              :style="{ paddingLeft: `${12 + Math.min(chapter.level, 5) * 16}px` }"
              @click="emit('select', bookId, chapter.index)"
            >
              <span class="chapter-number">{{ chapter.index + 1 }}</span
              >{{ chapter.title }}
            </button>
          </li>
        </ol>
        <p v-if="!matches.length" class="chapter-hint" role="status">没有匹配的章节。</p>
        <p class="chapter-hint">隐藏时选章会保存位置，显示阅读窗口后从该章开始。</p>
      </template>
    </details>
    <p v-else-if="!loading" class="chapter-hint">
      未找到独立的章节标题，可以继续逐页阅读。TXT 支持“第 X 章”“Chapter 1”等常见标题。
    </p>
  </section>
</template>

<style scoped>
.chapter-directory {
  margin: 18px 0;
  border-top: 1px solid #c4ccbd;
  border-bottom: 1px solid #c4ccbd;
  padding: 16px 0;
}
.chapter-current {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  font-size: 12px;
  color: #58665b;
}
.chapter-current strong {
  display: block;
  margin-top: 5px;
  font-size: 15px;
  color: #203d32;
  overflow-wrap: anywhere;
}
.chapter-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
details {
  margin-top: 16px;
}
summary {
  cursor: pointer;
  color: #203d32;
  font-size: 13px;
}
summary span {
  color: #58665b;
  font-size: 11px;
  margin-left: 8px;
}
.chapter-search {
  box-sizing: border-box;
  width: 100%;
  margin: 12px 0 8px;
  padding: 9px 12px;
  border: 1px solid #adb9a9;
  border-radius: 4px;
  background: transparent;
  color: #203d32;
  font: inherit;
}
.chapter-list {
  list-style: none;
  padding: 0;
  margin: 0;
  max-height: 260px;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.chapter-list button {
  width: 100%;
  display: flex;
  gap: 12px;
  text-align: left;
  padding: 10px 12px;
  border: 0;
  border-radius: 3px;
  color: #203d32;
  background: transparent;
  font-size: 13px;
  line-height: 1.5;
  white-space: normal;
  overflow-wrap: anywhere;
}
.chapter-list button:hover:enabled {
  background: #e7eadd;
  color: #203d32;
}
.chapter-list button[aria-current] {
  background: #dce4d3;
  font-weight: 600;
}
.chapter-number {
  flex: 0 0 24px;
  color: #937b5d;
  font-variant-numeric: tabular-nums;
}
.chapter-hint {
  font-size: 11px;
  line-height: 1.7;
  color: #58665b;
  margin: 10px 0 0;
}
@media (max-width: 620px) {
  .chapter-current {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
