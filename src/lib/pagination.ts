export interface Typography {
  fontFamily: string;
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  width: number;
  lines: number;
}
export interface Page {
  start: number;
  end: number;
  text: string;
}

// Same text styles on the visible page and measurement node. Offsets are UTF-16.
export function textStyle(settings: Typography) {
  return {
    fontFamily: settings.fontFamily,
    fontSize: `${settings.fontSize}px`,
    lineHeight: `${settings.fontSize * settings.lineHeight}px`,
    letterSpacing: `${settings.letterSpacing}px`,
    whiteSpace: 'break-spaces' as const,
    overflowWrap: 'anywhere' as const,
    wordBreak: 'normal' as const,
    fontWeight: '400',
    margin: '0',
    padding: '0',
  };
}
const segmenter = new Intl.Segmenter('zh', { granularity: 'grapheme' });

export class Paginator {
  private measure: HTMLDivElement;
  private cache = new Map<number, Page>();
  private previousEnds = new Map<number, number>();
  private lineHeight: number;
  private availableHeight: number;
  constructor(
    private text: string,
    settings: Typography,
  ) {
    this.lineHeight = settings.fontSize * settings.lineHeight;
    this.availableHeight = this.lineHeight * settings.lines;
    this.measure = document.createElement('div');
    Object.assign(this.measure.style, textStyle(settings), {
      position: 'fixed',
      left: '-100000px',
      top: '0',
      visibility: 'hidden',
      pointerEvents: 'none',
      width: `${Math.max(1, settings.width - 40)}px`,
      height: 'auto',
      minHeight: '0',
      maxHeight: 'none',
      border: '0',
    });
    this.measure.setAttribute('aria-hidden', 'true');
    document.body.append(this.measure);
  }
  destroy() {
    this.measure.remove();
    this.cache.clear();
    this.previousEnds.clear();
  }
  private fits(value: string) {
    // A sentinel gives a trailing newline its real empty line box.
    this.measure.textContent = value.endsWith('\n') ? `${value}\u200b` : value;
    return this.measure.getBoundingClientRect().height <= this.availableHeight + 0.1;
  }
  page(start: number): Page {
    if (!Number.isInteger(start) || start < 0 || start > this.text.length)
      throw Error('阅读位置无效');
    const cached = this.cache.get(start);
    if (cached) return cached;
    if (start === this.text.length) return { start, end: start, text: '' };
    let length = 2048;
    let boundaries: number[] = [];
    while (true) {
      const limit = Math.min(this.text.length, start + length);
      const candidate = this.text.slice(start, limit);
      boundaries = [...segmenter.segment(candidate)].map((s) => start + s.index);
      if (limit === this.text.length) boundaries.push(limit);
      // Drop a possibly truncated final grapheme when there is more source text.
      const last = boundaries.at(-1) ?? start;
      if (last > start && (!this.fits(this.text.slice(start, last)) || limit === this.text.length))
        break;
      length *= 2;
    }
    let low = 0,
      high = boundaries.length - 1;
    while (low < high) {
      const mid = Math.ceil((low + high) / 2);
      if (this.fits(this.text.slice(start, boundaries[mid]!))) low = mid;
      else high = mid - 1;
    }
    const end = boundaries[Math.max(1, low)] ?? this.text.length;
    const page = { start, end, text: this.text.slice(start, end) };
    this.cache.set(start, page);
    this.previousEnds.set(end, start);
    // Keep only a bounded working set; never lay out an entire novel in the DOM.
    while (this.cache.size > 8) this.cache.delete(this.cache.keys().next().value!);
    while (this.previousEnds.size > 32)
      this.previousEnds.delete(this.previousEnds.keys().next().value!);
    return page;
  }
  previous(start: number): number | null {
    if (start === 0) return null;
    const known = this.previousEnds.get(start);
    if (known !== undefined) return known;
    // After reflow/restart choose the largest complete suffix that fits before the
    // anchor. Do not re-page from the beginning of a multi-megabyte novel.
    let length = 2048;
    let candidates: number[] = [];
    while (true) {
      const from = Math.max(0, start - length);
      const slice = this.text.slice(from, start);
      candidates = [...segmenter.segment(slice)].map((s) => from + s.index);
      if (from > 0) candidates.shift(); // First cluster may have been truncated.
      candidates.push(start);
      if (from === 0 || !this.fits(this.text.slice(candidates[0], start))) break;
      length *= 2;
    }
    let low = 0,
      high = candidates.length - 1;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (this.fits(this.text.slice(candidates[mid]!, start))) high = mid;
      else low = mid + 1;
    }
    const previous = candidates[Math.min(low, candidates.length - 2)] ?? 0;
    // A backward page must end exactly at the old anchor, even if punctuation
    // reflow would otherwise fit one extra character from the following page.
    this.cache.set(previous, {
      start: previous,
      end: start,
      text: this.text.slice(previous, start),
    });
    this.previousEnds.set(start, previous);
    return previous;
  }
}
