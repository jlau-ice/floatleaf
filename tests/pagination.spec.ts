import { test, expect } from '@playwright/test';
test('actual DOM pagination preserves all graphemes across mixed text, blank lines and long words', async ({
  page,
}) => {
  await page.goto('/');
  const result = await page.evaluate(async () => {
    const { Paginator } = await import('/src/lib/pagination.ts');
    const text = (
      '春风吹过窗台。「下一页」应该完整。\n\nHello world, punctuation! ' +
      'unbroken'.repeat(90) +
      '\n😀👨‍👩‍👧‍👦e\u0301🇨🇳  结束\n'
    ).repeat(7);
    const graphemeBoundaries = new Set(
      [...new Intl.Segmenter('zh', { granularity: 'grapheme' }).segment(text)].map((s) => s.index),
    );
    graphemeBoundaries.add(text.length);
    const checks: { count: number; exact: boolean; boundaries: boolean; backwards: boolean }[] = [];
    for (const fontFamily of ['PingFang SC, sans-serif', 'Georgia, serif']) {
      for (const lines of [1, 2, 8]) {
        const p = new Paginator(text, {
          fontFamily,
          fontSize: 18,
          lineHeight: 1.5,
          letterSpacing: 0.5,
          width: 320,
          lines,
        });
        let start = 0,
          combined = '',
          boundaries = true,
          backwards = true,
          count = 0;
        while (start < text.length) {
          const value = p.page(start);
          if (value.end <= start) throw Error('page did not advance');
          boundaries &&= graphemeBoundaries.has(value.end);
          combined += value.text;
          const previous = p.previous(value.end);
          backwards &&= previous === start && p.page(previous!).end === value.end;
          start = value.end;
          count++;
        }
        checks.push({ count, exact: combined === text, boundaries, backwards });
        p.destroy();
      }
    }
    return checks;
  });
  for (const check of result) {
    expect(check.count).toBeGreaterThan(1);
    expect(check.exact && check.boundaries && check.backwards).toBe(true);
  }
});

test('reflow keeps the reading anchor and reconstructs a gap-free preceding page', async ({
  page,
}) => {
  await page.goto('/');
  const result = await page.evaluate(async () => {
    const { Paginator } = await import('/src/lib/pagination.ts');
    const text = (
      '正文：中文 English 👩‍💻 标点。\n\n下一段很长 ' +
      'longword'.repeat(30) +
      '\n'
    ).repeat(40);
    const initial = {
      fontFamily: 'PingFang SC, sans-serif',
      fontSize: 18,
      lineHeight: 1.5,
      letterSpacing: 0,
      width: 600,
      lines: 2,
    };
    const first = new Paginator(text, initial);
    let anchor = 0;
    for (let i = 0; i < 12; i++) anchor = first.page(anchor).end;
    first.destroy();
    const next = new Paginator(text, { ...initial, width: 220, fontSize: 30, lines: 1 });
    const current = next.page(anchor),
      prev = next.previous(anchor)!;
    const preceding = next.page(prev);
    const result = { anchor, current, preceding, expected: text.slice(prev, current.end) };
    next.destroy();
    return result;
  });
  expect(result.current.start).toBe(result.anchor);
  expect(result.preceding.end).toBe(result.anchor);
  expect(result.preceding.text + result.current.text).toBe(result.expected);
});

test('large text lays out a bounded DOM window and cleans up measuring nodes', async ({ page }) => {
  await page.goto('/');
  const result = await page.evaluate(async () => {
    const { Paginator } = await import('/src/lib/pagination.ts');
    const text = '测试内容 English 😀\n'.repeat(200000);
    const start = performance.now();
    const p = new Paginator(text, {
      fontFamily: 'sans-serif',
      fontSize: 12,
      lineHeight: 1,
      letterSpacing: -1,
      width: 1600,
      lines: 12,
    });
    const page = p.page(0),
      last = p.page(text.length);
    const measuring = document.querySelector('[aria-hidden=true][style*="100000"]')!;
    const nodeLength = measuring.textContent!.length;
    p.destroy();
    return {
      pageEnd: page.end,
      last,
      nodeLength,
      remaining: document.querySelectorAll('[aria-hidden=true][style*="100000"]').length,
      ms: performance.now() - start,
    };
  });
  expect(result.pageEnd).toBeGreaterThan(0);
  expect(result.nodeLength).toBeLessThan(20000);
  expect(result.last.text).toBe('');
  expect(result.remaining).toBe(0);
  console.log('Large-text layout diagnostic (not native latency):', Math.round(result.ms), 'ms');
});
