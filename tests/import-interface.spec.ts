import { expect, test } from '@playwright/test';

// IPC mock isolates asynchronous UI behavior; native import is covered separately.
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
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
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        transformCallback: () => 1,
        unregisterCallback: () => {},
        invoke: async (command: string, args: Record<string, string> = {}) => {
          if (command === 'reader_state')
            return {
              settings,
              mode: 'hold',
              visible: false,
              positionLocked: true,
              revision: 1,
              error: null,
              reader: { bookId: null, total: 0, anchor: 0, end: 0, ready: false },
            };
          if (command === 'library') return { books: [], currentBookId: null, settings };
          if (command === 'data_directory') return '/test-data';
          if (command === 'log_info')
            return { directory: '/test-data/logs', version: '0.1.1', error: null };
          if (command === 'choose_txt') return '/synthetic.txt';
          if (command === 'inspect_txt') {
            await new Promise((r) => setTimeout(r, args.encoding === 'gbk' ? 180 : 10));
            return {
              title: '合成文本',
              encoding: args.encoding,
              preview: args.encoding + ' 正确预览',
              warning: null,
              utf16Length: 8,
            };
          }
          if (command === 'import_txt') throw '源文件已移动，请重新选择文件';
          return 1;
        },
      },
    });
  });
  await page.goto('/');
  await page.getByRole('button', { name: '＋ 导入小说' }).click();
});

test('latest encoding preview wins when decoding finishes out of order', async ({ page }) => {
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('.import-preview')).toContainText('auto 正确预览');
  await dialog.getByRole('combobox').selectOption('gbk');
  await dialog.getByRole('combobox').selectOption('utf8');
  await expect(dialog.locator('.import-preview')).toContainText('utf8 正确预览');
  await page.waitForTimeout(250);
  await expect(dialog.locator('.import-preview')).toContainText('utf8 正确预览');
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
});

test('import failure stays visible inside the modal and focus cannot reach background controls', async ({
  page,
}) => {
  const dialog = page.getByRole('dialog');
  await expect(dialog.getByRole('button', { name: '确认导入' })).toBeEnabled();
  for (let i = 0; i < 6; i++) {
    await page.keyboard.press('Tab');
    expect(
      await page.evaluate(
        () =>
          document.activeElement === document.body || !!document.activeElement?.closest('dialog'),
      ),
    ).toBe(true);
  }
  await dialog.getByRole('button', { name: '确认导入' }).click();
  await expect(dialog.getByRole('alert')).toHaveText('源文件已移动，请重新选择文件');
  await expect(dialog).toBeVisible();
});
