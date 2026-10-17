import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onMounted, onUnmounted, ref } from 'vue';

export interface ReaderState {
  mode: 'hold' | 'toggle' | 'pinned';
  visible: boolean;
  emergencyLatched: boolean;
  editingLayout: boolean;
  positionLocked: boolean;
  revision: number;
  shortcuts: string[];
  error: string | null;
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
      const unlisten = await listen<ReaderState>('reader-state', e => accept(e.payload));
      if (!alive) { unlisten(); return; }
      dispose = unlisten;
      accept(await invoke<ReaderState>('reader_state'));
    } catch (e) { error.value = String(e); }
  });
  onUnmounted(() => { alive = false; dispose?.(); });
  async function action(action: string) {
    error.value = '';
    try { await invoke('reader_action', { action }); }
    catch (e) { error.value = String(e); }
  }
  return { state, error, action };
}
