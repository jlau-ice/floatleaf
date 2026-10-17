import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onMounted, onUnmounted, ref } from 'vue';

export interface Bindings {
  display: string;
  emergency: string;
  previous: string;
  next: string;
  auto: string;
}
export interface ReadingSettings {
  fontFamily: string;
  fontSize: number;
  color: string;
  textOpacity: number;
  backgroundOpacity: number;
  lineHeight: number;
  letterSpacing: number;
  width: number;
  lines: number;
  intervalSeconds: number;
  mode: 'hold' | 'toggle' | 'pinned';
  positionLocked: boolean;
  shortcuts: Bindings;
}
export interface Book {
  id: string;
  title: string;
  encoding: string;
  utf16Length: number;
  anchor: number;
  addedAt: number;
}
export interface Chapter {
  title: string;
  anchor: number;
  level: number;
}
export interface Library {
  books: Book[];
  currentBookId: string | null;
  settings: ReadingSettings;
}
export interface ReaderState {
  mode: 'hold' | 'toggle' | 'pinned';
  visible: boolean;
  emergencyLatched: boolean;
  editingLayout: boolean;
  positionLocked: boolean;
  revision: number;
  placement: { monitor: string | null; x: number; y: number } | null;
  error: string | null;
  settings: ReadingSettings;
  reader: {
    bookId: string | null;
    anchor: number;
    total: number;
    end: number;
    previous: number | null;
    layoutVersion: number;
    ready: boolean;
    autoRunning: boolean;
    remainingMs: number;
    intervalMs: number;
  };
}
export const desktop = isTauri();
export function useReader() {
  const state = ref<ReaderState>();
  const error = ref('');
  let dispose: (() => void) | undefined;
  let alive = true;
  function accept(value: ReaderState) {
    if (!state.value || value.revision >= state.value.revision) state.value = value;
  }
  onMounted(async () => {
    if (!desktop) return;
    try {
      const unlisten = await listen<ReaderState>('reader-state', (e) => accept(e.payload));
      if (!alive) {
        unlisten();
        return;
      }
      dispose = unlisten;
      accept(await invoke<ReaderState>('reader_state'));
    } catch (e) {
      error.value = String(e);
    }
  });
  onUnmounted(() => {
    alive = false;
    dispose?.();
  });
  async function action(action: string) {
    error.value = '';
    try {
      await invoke('reader_action', { action });
    } catch (e) {
      error.value = String(e);
    }
  }
  return { state, error, action };
}
