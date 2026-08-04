/**
 * Минимальный разбор и сборка Fluent-каталогов для табличного редактора.
 *
 * Полноценный парсер здесь не нужен и был бы лишней зависимостью: правится
 * плоский список `ключ = значение`, а многострочные записи (селекторы
 * множественного числа) редактор показывает как есть, одним блоком.
 */

export interface FtlEntry {
  key: string
  value: string
}

/** Ключ верхнего уровня начинается с буквы в первой колонке. */
const KEY_LINE = /^([a-zA-Z][\w-]*)\s*=\s*(.*)$/

export function parseFtl(text: string): FtlEntry[] {
  const out: FtlEntry[] = []
  let current: FtlEntry | null = null

  for (const line of text.split('\n')) {
    const match = KEY_LINE.exec(line)
    if (match) {
      if (current) out.push(current)
      current = { key: match[1], value: match[2] }
      continue
    }
    // Продолжение записи: отступ или пустая строка внутри блока.
    if (current && (line.startsWith(' ') || line.startsWith('\t'))) {
      current.value += '\n' + line
      continue
    }
    if (current) {
      out.push(current)
      current = null
    }
  }
  if (current) out.push(current)
  return out
}

export function buildFtl(entries: FtlEntry[]): string {
  return entries
    .filter(e => e.key && e.value.trim())
    .map(e => `${e.key} = ${e.value}`)
    .join('\n')
    .concat('\n')
}

/** Многострочная запись — селектор; в таблице её правят в textarea. */
export function isMultiline(value: string) {
  return value.includes('\n')
}
