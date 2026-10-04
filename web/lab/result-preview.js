// Imported results are untrusted data. Bound display work without changing exports.
export const MAX_PREVIEW_CHARS = 100000;
export function resultPreview(value, limit = MAX_PREVIEW_CHARS) {
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_PREVIEW_CHARS) throw new Error('Invalid preview limit.');
  const parts = [];
  let remaining = limit;
  function* tokens(item, depth) {
    if (item === null || typeof item !== 'object') {
      // Avoid encoding an entire large string just to display its beginning.
      yield JSON.stringify(typeof item === 'string' ? item.slice(0, remaining + 1) : item);
      return;
    }
    const array = Array.isArray(item), keys = Object.keys(item);
    yield array ? '[' : '{';
    for (let i = 0; i < keys.length; i++) {
      yield (i ? ',\n' : '\n') + '  '.repeat(depth + 1);
      if (!array) yield JSON.stringify(keys[i].slice(0, remaining + 1)) + ': ';
      yield* tokens(item[keys[i]], depth + 1);
    }
    if (keys.length) yield '\n' + '  '.repeat(depth);
    yield array ? ']' : '}';
  }
  for (const token of tokens(value, 0)) {
    if (token.length > remaining) {
      parts.push(token.slice(0, remaining));
      return parts.join('') + '\n… Preview shortened. Download result JSON for the complete result.';
    }
    parts.push(token); remaining -= token.length;
  }
  return parts.join('');
}
