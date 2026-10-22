import { expect, test } from '@playwright/test';

// Native registration stays in Rust; this fixture exercises recording and save feedback.
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
      captureProtection: false,
      shortcuts: {
        display: 'Control+Alt+Space',
        emergency: 'Control+Alt+Escape',
        previous: 'Control+Alt+ArrowLeft',
        next: 'Control+Alt+ArrowRight',
        auto: 'Control+Alt+KeyP',
      },
    };
    const state = {
      captureProtectionSupported: true,
      settings,
      mode: 'hold',
      visible: false,
      positionLocked: true,
      revision: 1,
      error: null,
      shortcutBlockedBy: null as string | null,
      reader: { bookId: null, total: 0, anchor: 0, end: 0, ready: false },
    };
    const fixture = {
      saves: [] as (typeof settings)[],
      failSave: false,
      inputAccess: null as boolean | null,
      permissionRequests: 0,
      setCaptureSupport(supported: boolean) {
        state.captureProtectionSupported = supported;
        state.revision++;
        callbacks.get(stateCallback)?.({ payload: structuredClone(state) });
      },
      setBlockedBy(executable: string | null) {
        state.shortcutBlockedBy = executable;
        state.revision++;
        callbacks.get(stateCallback)?.({ payload: structuredClone(state) });
      },
    };
    const callbacks = new Map<number, (event: unknown) => void>();
    let nextCallback = 0;
    let stateCallback = 0;
    Object.assign(window, {
      isTauri: true,
      shortcutFixture: fixture,
      __TAURI_INTERNALS__: {
        transformCallback: (callback: (event: unknown) => void) => {
          callbacks.set(++nextCallback, callback);
          return nextCallback;
        },
        unregisterCallback: () => {},
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          if (command === 'plugin:event|listen' && args.event === 'reader-state')
            stateCallback = args.handler as number;
          if (command === 'reader_state') return structuredClone(state);
          if (command === 'library') return { books: [], currentBookId: null, settings };
          if (command === 'data_directory') return '/test-data';
          if (command === 'log_info')
            return { directory: '/test-data/logs', version: 'test', error: null };
          if (command === 'input_access') return fixture.inputAccess;
          if (command === 'open_input_settings') fixture.permissionRequests++;
          if (command === 'save_settings') {
            // IPC serializes nested Vue proxies before Rust receives settings.
            const saved = JSON.parse(JSON.stringify(args.settings)) as typeof settings;
            fixture.saves.push(saved);
            if (fixture.failSave) throw '组合键已被占用，原快捷键保留，请换一组。';
            state.settings = saved;
            state.revision++;
            callbacks.get(stateCallback)?.({ payload: structuredClone(state) });
          }
          return 1;
        },
      },
    });
  });
  await page.goto('/');
  await page.getByRole('button', { name: '快捷键', exact: true }).click();
});

test('capture protection saves through the normal settings command and is Windows only', async ({
  page,
}) => {
  const toggle = page.getByRole('switch', { name: '截图 / 录屏保护', exact: true });
  await expect(toggle).toHaveAttribute('aria-checked', 'false');
  await expect(page.locator('#capture-protection-hint')).toContainText('设置页不受保护');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-checked', 'true');
  await page.getByRole('button', { name: '固定', exact: true }).click();
  await expect(toggle).toHaveAttribute('aria-checked', 'true');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-checked', 'false');
  expect(
    await page.evaluate(() =>
      (
        window as unknown as {
          shortcutFixture: { saves: { captureProtection: boolean; mode: string }[] };
        }
      ).shortcutFixture.saves.map((s) => [s.captureProtection, s.mode]),
    ),
  ).toEqual([
    [true, 'hold'],
    [true, 'pinned'],
    [false, 'pinned'],
  ]);
  await page.evaluate(() =>
    (
      window as unknown as { shortcutFixture: { setCaptureSupport(supported: boolean): void } }
    ).shortcutFixture.setCaptureSupport(false),
  );
  await expect(toggle).toHaveCount(0);
});

test('background permission is requested only by the user and denial keeps the guidance visible', async ({
  page,
}) => {
  const permission = page.getByRole('complementary', { name: '后台快捷键权限' });
  await expect(permission).toHaveCount(0); // Windows has no macOS permission requirement.
  await page.evaluate(() => {
    (
      window as unknown as { shortcutFixture: { inputAccess: boolean | null } }
    ).shortcutFixture.inputAccess = false;
    window.dispatchEvent(new Event('focus'));
  });
  await expect(permission).toBeVisible();
  await expect(permission).toContainText('切换到其他软件');
  await expect(permission).toContainText('当前是开发模式');
  await expect(permission).toContainText('独立调试');
  const requests = () =>
    page.evaluate(
      () =>
        (window as unknown as { shortcutFixture: { permissionRequests: number } }).shortcutFixture
          .permissionRequests,
    );
  expect(await requests()).toBe(0);
  await page.getByRole('button', { name: '设置输入监控', exact: true }).click();
  expect(await requests()).toBe(1);
  await page.getByRole('button', { name: '重新检查', exact: true }).click();
  await expect(permission).toBeVisible();
  await page.evaluate(() => {
    (
      window as unknown as { shortcutFixture: { inputAccess: boolean } }
    ).shortcutFixture.inputAccess = true;
    window.dispatchEvent(new Event('focus'));
  });
  await expect(permission).toHaveCount(0);
  await expect(page.getByText('输入监控已开启；可在其他软件前台使用快捷键。')).toBeVisible();
  expect(await requests()).toBe(1);
});

test('recording is explicit and a captured shortcut only takes effect after saving', async ({
  page,
}) => {
  const recorder = page.locator('.shortcut-recorder').first();
  const save = page.getByRole('button', { name: '保存快捷键', exact: true });
  await expect(page.getByRole('button', { name: /录制快捷键，当前/ })).toHaveCount(5);
  await expect(save).toBeDisabled();
  await recorder.focus();
  await page.keyboard.press('Control+Alt+KeyR');
  await expect(recorder).toContainText('Ctrl + Alt + 空格');
  await recorder.press('Enter');
  await expect(recorder).toHaveAttribute('aria-pressed', 'true');
  await expect(recorder).toContainText('请按下组合键');
  await expect(save).toBeDisabled();
  await page.keyboard.press('Control+Alt+KeyR');
  await expect(recorder).toContainText('Ctrl + Alt + R');
  await expect(page.locator('.shortcut-unsaved')).toHaveText('尚未保存');
  await expect(page.locator('#shortcut-feedback')).toContainText('点击「保存快捷键」后生效');
  expect(
    await page.evaluate(
      () => (window as unknown as { shortcutFixture: { saves: unknown[] } }).shortcutFixture.saves,
    ),
  ).toHaveLength(0);
  await save.click();
  await expect(page.locator('#shortcut-feedback')).toContainText('快捷键已保存');
  await expect(save).toBeDisabled();
  expect(
    await page.evaluate(
      () =>
        (window as unknown as { shortcutFixture: { saves: { shortcuts: { display: string } }[] } })
          .shortcutFixture.saves[0]?.shortcuts.display,
    ),
  ).toBe('Control+Alt+KeyR');
});

test('Escape, Tab and clicking another control cancel recording without replacing a shortcut', async ({
  page,
}) => {
  const recorder = page.locator('.shortcut-recorder').first();
  await recorder.click();
  await page.keyboard.press('KeyR');
  await expect(page.locator('#shortcut-feedback')).toContainText('请按住 Ctrl');
  await expect(recorder).toHaveAttribute('aria-pressed', 'true');
  await page.keyboard.press('Escape');
  await expect(recorder).toContainText('Ctrl + Alt + 空格');
  await expect(page.locator('#shortcut-feedback')).toContainText('已取消录制');
  await recorder.click();
  await page.keyboard.press('Tab');
  await expect(page.locator('.shortcut-recorder').nth(1)).toBeFocused();
  await expect(recorder).toHaveAttribute('aria-pressed', 'false');
  await recorder.click();
  await page.getByRole('button', { name: '文字与布局', exact: true }).click();
  await page.getByRole('button', { name: '快捷键', exact: true }).click();
  await expect(recorder).toHaveAttribute('aria-pressed', 'false');
  await expect(recorder).toContainText('Ctrl + Alt + 空格');
  await expect(page.getByRole('button', { name: '保存快捷键', exact: true })).toBeDisabled();
});

test('registration failure keeps the recorded draft editable and gives local feedback', async ({
  page,
}) => {
  await page.evaluate(() => {
    (window as unknown as { shortcutFixture: { failSave: boolean } }).shortcutFixture.failSave =
      true;
  });
  const recorder = page.locator('.shortcut-recorder').first();
  await recorder.click();
  await page.keyboard.press('Control+Alt+Escape');
  await expect(recorder).toContainText('Ctrl + Alt + Esc');
  await page.getByRole('button', { name: '保存快捷键', exact: true }).click();
  await expect(page.locator('#shortcut-feedback')).toContainText('原快捷键保留');
  await expect(page.locator('.shortcut-unsaved')).toHaveText('尚未保存');
  await expect(recorder).toContainText('Ctrl + Alt + Esc');
  await expect(recorder).toBeEnabled();
});

test('Windows shortcut guidance follows runtime state without requesting elevation', async ({
  page,
}) => {
  const warning = page.getByRole('complementary', { name: 'Windows 快捷键权限' });
  await expect(warning).toHaveCount(0);
  await page.evaluate(() => {
    (
      window as unknown as { shortcutFixture: { setBlockedBy(name: string | null): void } }
    ).shortcutFixture.setBlockedBy('PddWorkbench.exe');
  });
  await expect(warning).toBeVisible();
  await expect(warning).toContainText('PddWorkbench.exe');
  await expect(warning).toContainText('先从托盘退出浮页');
  await expect(warning).toContainText('浮页不会自动提权');
  await expect(warning.getByRole('button')).toHaveCount(0);
  await page.evaluate(() => {
    (
      window as unknown as { shortcutFixture: { setBlockedBy(name: string | null): void } }
    ).shortcutFixture.setBlockedBy(null);
  });
  await expect(warning).toHaveCount(0);
});
