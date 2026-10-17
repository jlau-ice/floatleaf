import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const chapters = [
      { title: '第一章 出发', anchor: 0, level: 0 },
      { title: '第二章 山路', anchor: 100, level: 0 },
      { title: '第三章 回家😀', anchor: 200, level: 1 },
    ];
    const settings = {
      fontFamily: 'sans-serif',
      fontSize: 18,
      color: '#ffffff',
      textOpacity: 0.7,
      backgroundOpacity: 0,
      lineHeight: 1.5,
      letterSpacing: 0,
      width: 600,
      lines: 2,
      intervalSeconds: 8,
      mode: 'hold',
      positionLocked: true,
      shortcuts: {
        display: 'Control+Alt+Space',
        emergency: 'Control+Alt+Escape',
        previous: 'Control+Alt+ArrowLeft',
        next: 'Control+Alt+ArrowRight',
        auto: 'Control+Alt+KeyP',
      },
    };
    const book = {
      id: 'fixture',
      title: '合成章节',
      encoding: 'EPUB',
      utf16Length: 300,
      anchor: 120,
      addedAt: 0,
    };
    const state = {
      settings,
      mode: 'hold',
      visible: false,
      positionLocked: true,
      revision: 1,
      error: null,
      reader: {
        bookId: book.id,
        total: 300,
        anchor: 120,
        end: 150,
        previous: 80,
        ready: true,
        autoRunning: false,
      },
    };
    const requests: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      chapterRequests: requests,
      __TAURI_INTERNALS__: {
        transformCallback: () => 1,
        unregisterCallback: () => {},
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          if (command === 'reader_state') return state;
          if (command === 'library') return { books: [book], currentBookId: book.id, settings };
          if (command === 'book_chapters') return chapters;
          if (command === 'select_chapter') {
            requests.push(args);
            return;
          }
          if (command === 'data_directory') return '/test-data';
          if (command === 'log_info')
            return { directory: '/test-data/logs', version: '0.2.0', error: null };
          return 1;
        },
      },
    });
  });
  await page.goto('/');
});

test('hidden reader can choose filtered chapters using the original chapter index', async ({
  page,
}) => {
  const directory = page.getByRole('region', { name: '章节导航' });
  await expect(directory).toContainText('第二章 山路');
  await directory.locator('summary').click();
  await page.getByRole('searchbox', { name: '搜索章节' }).fill('回家');
  await expect(page.getByRole('list', { name: '章节列表' }).getByRole('button')).toHaveCount(1);
  await page.getByRole('button', { name: '3 第三章 回家😀' }).click();
  expect(
    await page.evaluate(
      () => (window as unknown as { chapterRequests: unknown[] }).chapterRequests,
    ),
  ).toEqual([{ id: 'fixture', index: 2 }]);
  await page.getByRole('searchbox', { name: '搜索章节' }).fill('不存在');
  await expect(directory.getByRole('status')).toHaveText('没有匹配的章节。');
});

test('chapter navigation stays enabled while page turning is paused by hiding', async ({
  page,
}) => {
  await expect(page.getByRole('button', { name: '← 上一页', exact: true })).toBeDisabled();
  await expect(page.getByRole('button', { name: '← 上一章', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '← 上一章', exact: true }).click();
  await page.getByRole('button', { name: '下一章 →', exact: true }).click();
  expect(
    await page.evaluate(
      () => (window as unknown as { chapterRequests: unknown[] }).chapterRequests,
    ),
  ).toEqual([
    { id: 'fixture', index: 0 },
    { id: 'fixture', index: 2 },
  ]);
});
