<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  desktop,
  useReader,
  type Book,
  type Library,
  type ReadingSettings,
  type Bindings,
} from '../lib/desktop';
const { state, error, action } = useReader();
const shelf = ref<Library>();
const busy = ref(false);
const activeTab = ref<'library' | 'appearance' | 'shortcuts'>('library');
const draft = ref<ReadingSettings>();
const saved = ref('');
const shortcutDraft = ref<Bindings>();
const shortcutLabels = {
  display: '显示 / 隐藏',
  emergency: '紧急隐藏',
  previous: '上一页',
  next: '下一页',
  auto: '自动翻页暂停 / 恢复',
};
function keyLabel(value: string) {
  return value
    .replaceAll('Control', 'Ctrl')
    .replaceAll('Key', '')
    .replaceAll('ArrowLeft', '←')
    .replaceAll('ArrowRight', '→')
    .replaceAll('Space', '空格')
    .replaceAll('Escape', 'Esc')
    .replaceAll('+', ' + ');
}
function recordShortcut(event: KeyboardEvent, key: keyof Bindings) {
  if (event.key === 'Tab' || event.repeat) return;
  event.preventDefault();
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key) || !shortcutDraft.value) return;
  shortcutDraft.value[key] = [
    event.ctrlKey && 'Control',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Super',
    event.code,
  ]
    .filter(Boolean)
    .join('+');
}
async function saveShortcuts() {
  if (shortcutDraft.value) await save({ shortcuts: { ...shortcutDraft.value } });
}
const removing = ref('');
const dataDirectory = ref('');
const logInfo = ref<{ directory: string; version: string; error: string | null }>();
const importPath = ref('');
const importDialog = ref<HTMLDialogElement>();
let importRequest = 0;
watch(importPath, async (path) => {
  if (path) {
    await nextTick();
    importDialog.value?.showModal();
  }
});
function cancelImport(event: Event) {
  if (busy.value) event.preventDefault();
  else importPath.value = '';
}
const encoding = ref('auto');
const importPreview = ref<{
  title: string;
  encoding: string;
  preview: string;
  warning: string | null;
  utf16Length: number;
}>();
const importError = ref('');
const current = computed(() => shelf.value?.books.find((b) => b.id === state.value?.reader.bookId));
const percent = computed(() =>
  state.value?.reader.total
    ? Math.round(
        ((state.value.reader.end || state.value.reader.anchor) / state.value.reader.total) * 100,
      )
    : 0,
);
const status = computed(() =>
  state.value?.emergencyLatched
    ? '已紧急隐藏 · 全部松开快捷键后重新触发'
    : state.value?.visible
      ? '阅读窗口已显示'
      : '使用显示快捷键开始阅读',
);
let unlisten: (() => void) | undefined;
let alive = true;
let lastSettings = '';
watch(
  () => state.value?.settings,
  (value) => {
    if (!value) return;
    const key = JSON.stringify(value);
    if (key !== lastSettings) {
      draft.value = { ...value };
      shortcutDraft.value = { ...value.shortcuts };
      lastSettings = key;
    }
  },
);
async function refresh() {
  shelf.value = await invoke<Library>('library');
}
onMounted(async () => {
  if (!desktop) return;
  try {
    const stop = await listen('library-changed', () => {
      refresh().catch((e) => (error.value = String(e)));
    });
    if (!alive) {
      stop();
      return;
    }
    unlisten = stop;
    await refresh();
    dataDirectory.value = await invoke<string>('data_directory');
    logInfo.value = await invoke('log_info');
  } catch (e) {
    error.value = String(e);
  }
});
onUnmounted(() => {
  alive = false;
  unlisten?.();
});
async function task(work: () => Promise<unknown>) {
  if (busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    await work();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
async function choose() {
  await task(async () => {
    const path = await invoke<string | null>('choose_txt');
    if (path) {
      importPath.value = path;
      encoding.value = 'auto';
      await inspect();
    }
  });
}
async function inspect() {
  const request = ++importRequest;
  importError.value = '';
  importPreview.value = undefined;
  try {
    const preview = await invoke<typeof importPreview.value>('inspect_txt', {
      path: importPath.value,
      encoding: encoding.value,
    });
    if (request === importRequest) importPreview.value = preview;
  } catch (e) {
    if (request === importRequest) importError.value = String(e);
  }
}
async function confirmImport() {
  await task(async () => {
    try {
      await invoke('import_txt', { path: importPath.value, encoding: encoding.value });
      importPath.value = '';
      importPreview.value = undefined;
      await refresh();
    } catch (e) {
      importError.value = String(e);
    }
  });
}
async function select(book: Book) {
  await task(() => invoke('select_book', { id: book.id }));
}
async function remove(book: Book) {
  if (removing.value !== book.id) {
    removing.value = book.id;
    return;
  }
  await task(async () => {
    await invoke('remove_book', { id: book.id });
    removing.value = '';
    await refresh();
  });
}
async function save(patch: Partial<ReadingSettings> = {}) {
  if (!draft.value) return;
  const settings = { ...draft.value, ...patch };
  saved.value = '';
  await task(async () => {
    await invoke('save_settings', { settings, selectMode: patch.mode !== undefined });
    saved.value = '已保存';
  });
}
const ranges: {
  key:
    | 'fontSize'
    | 'textOpacity'
    | 'backgroundOpacity'
    | 'lineHeight'
    | 'letterSpacing'
    | 'width'
    | 'lines'
    | 'intervalSeconds';
  label: string;
  min: number;
  max: number;
  step: number;
  unit: string;
}[] = [
  { key: 'fontSize', label: '字号', min: 12, max: 48, step: 1, unit: 'px' },
  { key: 'lines', label: '完整行数', min: 1, max: 12, step: 1, unit: '行' },
  { key: 'width', label: '阅读宽度', min: 200, max: 1600, step: 10, unit: 'px' },
  { key: 'lineHeight', label: '行高', min: 1, max: 2.2, step: 0.05, unit: '倍' },
  { key: 'letterSpacing', label: '字间距', min: -1, max: 8, step: 0.1, unit: 'px' },
  { key: 'textOpacity', label: '文字不透明度', min: 0.1, max: 1, step: 0.05, unit: '' },
  { key: 'backgroundOpacity', label: '背景不透明度', min: 0, max: 0.8, step: 0.05, unit: '' },
  { key: 'intervalSeconds', label: '自动翻页间隔', min: 2, max: 60, step: 1, unit: '秒' },
];
</script>
<template>
  <main class="workspace">
    <header>
      <span class="wordmark">FLOATLEAF <span class="cn-mark">浮页</span></span
      ><button class="secondary compact" :disabled="!desktop" @click="action('hide')">
        紧急隐藏
        <span class="key-hint">{{
          keyLabel(state?.settings.shortcuts.emergency || 'Control+Alt+Escape')
        }}</span>
      </button>
    </header>
    <section class="intro">
      <p class="eyebrow">留一点空间，给文字。</p>
      <h1>阅读，在一旁<span>。</span></h1>
      <p class="description">
        <kbd>{{ keyLabel(state?.settings.shortcuts.display || 'Control+Alt+Space') }}</kbd>
        显示文字，阅读与手边的事自在切换。
      </p>
    </section>
    <p v-if="!desktop" class="notice">
      这是网页预览。导入和悬浮阅读需要桌面程序，请使用 pnpm desktop:dev。
    </p>
    <p v-if="error || state?.error" class="notice error" role="alert">
      {{ error || state?.error }}
    </p>
    <nav class="tabs" aria-label="设置页面">
      <button :aria-selected="activeTab === 'library'" @click="activeTab = 'library'">
        我的书架 <span>{{ shelf?.books.length ?? 0 }}</span></button
      ><button :aria-selected="activeTab === 'appearance'" @click="activeTab = 'appearance'">
        文字与布局</button
      ><button :aria-selected="activeTab === 'shortcuts'" @click="activeTab = 'shortcuts'">
        快捷键</button
      ><span class="save-status" role="status">{{ saved }}</span>
    </nav>
    <section v-if="activeTab === 'library'" class="library">
      <div class="section-heading">
        <div>
          <h2>把一本书，放到桌面上</h2>
          <p>TXT · UTF-8 / GBK · 本地保存</p>
        </div>
        <button :disabled="!desktop || busy" @click="choose">＋ 导入小说</button>
      </div>
      <div v-if="!shelf?.books.length" class="empty-shelf">
        <span class="book-symbol" aria-hidden="true">页</span>
        <div>
          <h3>书架还空着</h3>
          <p>
            导入后会保存独立副本，移动原文件也能继续阅读。<br />支持 20 MiB 以内的
            TXT；导入前可以核对文字编码。
          </p>
        </div>
      </div>
      <ul v-else class="book-list" aria-label="已导入的小说">
        <li
          v-for="(book, index) in shelf.books"
          :key="book.id"
          :class="{ selected: current?.id === book.id }"
        >
          <button class="book-select" :disabled="busy" @click="select(book)">
            <span class="book-index">{{ String(index + 1).padStart(2, '0') }}</span
            ><span
              ><strong>{{ book.title }}</strong
              ><small
                >{{ book.encoding }} ·
                {{
                  current?.id === book.id
                    ? '正在阅读'
                    : `${Math.round((book.anchor / book.utf16Length) * 100)}%`
                }}</small
              ></span
            ></button
          ><button class="remove-book" :disabled="busy" @click="remove(book)">
            {{ removing === book.id ? '确认移除副本' : '移除' }}
          </button>
        </li>
      </ul>
      <div v-if="current" class="reading-controls">
        <div class="current-heading">
          <strong>{{ current.title }}</strong
          ><span>{{ percent }}%</span>
        </div>
        <progress :value="percent" max="100" aria-label="阅读进度" />
        <div class="actions">
          <button
            class="secondary"
            :disabled="!state?.visible || !state.reader.ready || state.reader.previous === null"
            @click="action('previous')"
          >
            ← 上一页</button
          ><button
            class="secondary"
            :disabled="
              !state?.visible || !state.reader.ready || state.reader.end >= state.reader.total
            "
            @click="action('next')"
          >
            下一页 →</button
          ><button
            :disabled="
              !state?.visible || !state.reader.ready || state.reader.end >= state.reader.total
            "
            @click="action('auto')"
          >
            {{ state?.reader.autoRunning ? '暂停自动翻页' : '自动翻页' }}</button
          ><span class="timer">{{
            state?.reader.autoRunning
              ? `${Math.ceil(state.reader.remainingMs / 1000)} 秒`
              : state?.reader.end === state?.reader.total
                ? '已到结尾'
                : '已暂停'
          }}</span>
        </div>
        <p class="hint">
          {{ keyLabel(state?.settings.shortcuts.previous || '') }} /
          {{ keyLabel(state?.settings.shortcuts.next || '') }} 翻页；{{
            keyLabel(state?.settings.shortcuts.auto || '')
          }}
          暂停或继续。隐藏期间不会改变进度。
        </p>
      </div>
    </section>
    <section v-else-if="activeTab === 'appearance' && draft" class="appearance">
      <div class="section-heading">
        <div>
          <h2>适合你的阅读方式</h2>
          <p>调整排版后保留阅读位置，高度随完整行数变化。</p>
        </div>
      </div>
      <div class="style-grid">
        <label class="font-field"
          >字体<input
            v-model="draft.fontFamily"
            :disabled="busy"
            list="font-options"
            @change="save()" /><datalist id="font-options">
            <option value="PingFang SC, Microsoft YaHei, sans-serif" />
            <option value="Songti SC, SimSun, serif" />
            <option value="Georgia, serif" />
            <option value="Menlo, Consolas, monospace" /></datalist></label
        ><label class="color-field"
          >文字颜色<input
            v-model="draft.color"
            type="color"
            :disabled="busy"
            @change="save()" /></label
        ><label v-for="range in ranges" :key="range.key"
          ><span
            >{{ range.label }} <output>{{ draft[range.key] }}{{ range.unit }}</output></span
          ><input
            v-model.number="draft[range.key]"
            type="range"
            :min="range.min"
            :max="range.max"
            :step="range.step"
            :disabled="busy"
            @change="save()"
        /></label>
      </div>
    </section>
    <section v-else-if="shortcutDraft" class="shortcut-settings">
      <h2>按下要使用的组合键</h2>
      <p class="hint">
        先点输入框，再按组合键。须包含 Ctrl、Alt 或 Super；支持字母、空格、方向键、Esc、F1～F12。Alt
        在 Mac 上是 ⌥，Super 是 ⌘ / Win。
      </p>
      <label v-for="(label, key) in shortcutLabels" :key="key" class="shortcut-field">
        {{ label
        }}<input
          :value="keyLabel(shortcutDraft[key])"
          readonly
          :disabled="busy"
          :aria-label="label"
          @keydown="recordShortcut($event, key)"
        />
      </label>
      <button :disabled="busy" @click="saveShortcuts">保存快捷键</button>
      <p class="hint">
        系统报告占用时会提示，原快捷键继续可用。macOS
        允许部分共享快捷键；如与其他应用同时响应，请换一组。保存后重新触发即可显示。
      </p>
    </section>
    <section class="controls">
      <div class="row">
        <div>
          <h2>显示方式</h2>
          <p>位置锁定独立于显示方式。</p>
        </div>
        <div class="segmented" aria-label="显示方式">
          <button
            v-for="item in [
              { id: 'hold', name: '按住' },
              { id: 'toggle', name: '切换' },
              { id: 'pinned', name: '固定' },
            ] as const"
            :key="item.id"
            :disabled="!desktop || busy"
            :aria-pressed="state?.mode === item.id"
            @click="save({ mode: item.id })"
          >
            {{ item.name }}
          </button>
        </div>
      </div>
      <div class="row">
        <div>
          <h2>阅读区域</h2>
          <p>阅读时鼠标穿透，编辑时可拖动和调整尺寸。</p>
        </div>
        <div class="actions">
          <button
            class="secondary"
            :disabled="!desktop || busy"
            @click="save({ positionLocked: !state?.positionLocked })"
          >
            {{ state?.positionLocked ? '解除位置锁定' : '锁定位置' }}</button
          ><button
            :disabled="!desktop || state?.positionLocked"
            @click="action(state?.editingLayout ? 'finish' : 'edit')"
          >
            {{ state?.editingLayout ? '完成布局' : '编辑布局' }}</button
          ><button class="secondary" :disabled="!desktop" @click="action('reset-position')">
            找回位置
          </button>
        </div>
      </div>
    </section>
    <footer>
      <span class="status-dot" :class="{ visible: state?.visible }" /><span role="status">{{
        status
      }}</span
      ><span class="footnote">关闭设置后，从托盘或菜单栏找回。</span>
    </footer>
    <details v-if="dataDirectory" class="storage-note">
      <summary>本地数据位置</summary>
      <p>{{ dataDirectory }}</p>
      <p>退出浮页后备份 floatleaf.sqlite3，即可保留小说、进度和设置。移除书籍不会删除原始 TXT。</p>
    </details>
    <details v-if="logInfo" class="storage-note">
      <summary>日志位置 · v{{ logInfo.version }}</summary>
      <p>{{ logInfo.directory }}</p>
      <p>遇到异常后，将此文件夹中的 .log 文件发给反馈接收人即可。日志不包含小说正文。</p>
      <p v-if="logInfo.error" class="notice error">日志不可用：{{ logInfo.error }}</p>
      <button class="secondary" :disabled="busy" @click="task(() => invoke('open_log_directory'))">
        打开日志文件夹
      </button>
    </details>
    <dialog
      v-if="importPath"
      ref="importDialog"
      class="modal-backdrop"
      aria-labelledby="import-title"
      @cancel="cancelImport"
    >
      <section class="import-dialog">
        <h2 id="import-title">核对导入文字</h2>
        <p>{{ importPreview?.title || 'TXT 小说' }}</p>
        <label
          >文件编码<select v-model="encoding" :disabled="busy" @change="inspect">
            <option value="auto">自动检测</option>
            <option value="utf8">UTF-8</option>
            <option value="gbk">GBK</option>
          </select></label
        >
        <p v-if="importError" class="notice error" role="alert">{{ importError }}</p>
        <p v-if="importPreview?.warning" class="notice">{{ importPreview.warning }}</p>
        <pre v-if="importPreview" class="import-preview">{{ importPreview.preview }}</pre>
        <p class="hint">确认文字正常后导入。相同内容会复用已有书籍和进度。</p>
        <div class="actions">
          <button class="secondary" :disabled="busy" @click="importPath = ''">取消</button
          ><button :disabled="!importPreview || busy" @click="confirmImport">
            {{ busy ? '正在导入…' : '确认导入' }}
          </button>
        </div>
      </section>
    </dialog>
  </main>
</template>
<style scoped>
.shortcut-settings {
  padding: 26px 0;
}
.shortcut-field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin: 14px 0;
  font-size: 13px;
}
.shortcut-field input {
  width: 300px;
  padding: 10px;
  font: inherit;
  color: inherit;
  border: 1px solid #a7b4a5;
  border-radius: 4px;
  background: #fffdf4;
}
.cn-mark {
  letter-spacing: 0.1em;
  margin-left: 16px;
  font-family: 'Songti SC', serif;
  font-size: 16px;
}
.key-hint {
  opacity: 0.6;
  margin-left: 10px;
  font-size: 10px;
}
.intro {
  padding-top: 22px;
  padding-bottom: 24px;
}
h1 {
  font-size: 42px;
}
.tabs {
  display: flex;
  gap: 20px;
  align-items: center;
  border-bottom: 1px solid #c8cec0;
}
.tabs button {
  background: transparent;
  color: #58665b;
  border: 0;
  border-radius: 0;
  padding: 12px 0;
}
.tabs button[aria-selected='true'] {
  border-bottom: 2px solid #203b32;
  color: #203b32;
}
.tabs span {
  font-size: 10px;
  margin-left: 5px;
}
.save-status {
  margin-left: auto !important;
  color: #58665b;
}
.section-heading {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 22px 0 12px;
  gap: 16px;
}
.section-heading p,
.hint {
  font-size: 11px;
  color: #58665b;
  line-height: 1.8;
  margin: 6px 0;
}
.empty-shelf {
  display: flex;
  gap: 22px;
  padding: 22px;
  border: 1px dashed #c8cec0;
  margin: 8px 0 24px;
}
.book-symbol {
  font-family: 'Songti SC', serif;
  color: #a9774e;
  font-size: 52px;
}
h3 {
  font-size: 14px;
  font-weight: 550;
  margin: 4px 0 10px;
}
.empty-shelf p {
  font-size: 12px;
  line-height: 1.8;
  color: #58665b;
  margin: 0;
}
.book-list {
  list-style: none;
  padding: 0;
  margin: 4px 0 18px;
  max-height: 220px;
  overflow-y: auto;
}
.book-list li {
  display: flex;
  border-bottom: 1px solid #dedfd2;
  align-items: center;
}
.book-list li.selected {
  background: #e6e9dc;
}
.book-select {
  display: flex;
  text-align: left;
  align-items: center;
  gap: 16px;
  border: 0;
  background: transparent;
  color: #203b32;
  flex: 1;
  white-space: normal;
}
.book-select:hover:enabled {
  background: #e1e5d7;
  color: #203b32;
}
.book-index {
  font-family: Georgia, serif;
  color: #8e7054;
  font-size: 22px;
}
.book-select strong {
  font-size: 13px;
  font-weight: 550;
  overflow-wrap: anywhere;
}
.book-select small {
  display: block;
  color: #58665b;
  margin-top: 5px;
  font-size: 10px;
}
.remove-book {
  border: 0;
  background: transparent;
  color: #796450;
  font-size: 10px;
}
.reading-controls {
  padding: 6px 0 20px;
}
.current-heading {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  font-size: 12px;
}
progress {
  display: block;
  width: 100%;
  height: 3px;
  margin: 12px 0 18px;
  accent-color: #527c5e;
}
.timer {
  font-size: 11px;
  align-self: center;
  color: #58665b;
}
.style-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px 30px;
  padding: 8px 0 24px;
}
.style-grid label {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 12px;
}
.style-grid label span {
  display: flex;
  justify-content: space-between;
}
output {
  color: #8e7054;
  font-variant-numeric: tabular-nums;
}
input,
select {
  font: inherit;
  color: #203b32;
  background: #faf9f2;
  border: 1px solid #bbc5b6;
  border-radius: 4px;
  padding: 7px;
  min-width: 0;
}
input[type='range'] {
  width: 100%;
  accent-color: #3d6450;
  padding: 0;
}
input[type='color'] {
  height: 34px;
  width: 100%;
}
.storage-note {
  margin-top: 22px;
  font-size: 10px;
  color: #70806f;
  overflow-wrap: anywhere;
}
.storage-note summary {
  cursor: pointer;
}
.modal-backdrop::backdrop {
  background: transparent;
}
.modal-backdrop {
  width: 100%;
  height: 100%;
  max-width: none;
  max-height: none;
  margin: 0;
  border: 0;
  color: inherit;
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 24px;
  background: #14271e80;
  z-index: 20;
}
.import-dialog {
  width: min(620px, 100%);
  max-height: 90vh;
  overflow: auto;
  background: #f4f2e9;
  padding: 28px;
  border-radius: 6px;
  box-shadow: 0 20px 70px #10281c40;
}
.import-dialog label {
  display: flex;
  align-items: center;
  gap: 18px;
  font-size: 12px;
}
.import-preview {
  padding: 18px;
  max-height: 240px;
  overflow: auto;
  background: #e8ebdf;
  color: #203b32;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font:
    13px/1.9 'PingFang SC',
    sans-serif;
}
.import-dialog > p {
  overflow-wrap: anywhere;
  font-size: 12px;
}
.import-dialog > .actions {
  justify-content: flex-end;
}
@media (max-width: 700px) {
  .style-grid {
    gap: 18px;
  }
  .key-hint {
    display: none;
  }
  h1 {
    font-size: 36px;
  }
}
</style>
