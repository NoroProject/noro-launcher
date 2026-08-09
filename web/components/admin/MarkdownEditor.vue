<script setup lang="ts">
import DOMPurify from 'dompurify'
import { marked } from 'marked'

/**
 * Редактор markdown с панелью инструментов и превью — по образцу Modrinth.
 *
 * Раньше здесь стояла голая textarea: разметку приходилось помнить наизусть,
 * а увидеть результат можно было только опубликовав новость.
 */
const props = defineProps<{ modelValue: string, rows?: number }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()

const area = ref<HTMLTextAreaElement | null>(null)
const preview = ref(false)

/** Обёртка выделения: `**жирный**`, `# заголовок`, список и так далее. */
type Action =
  | { kind: 'wrap', before: string, after: string }
  | { kind: 'line', prefix: string }

const TOOLS: Array<{ icon: string, title: string, action: Action } | 'sep'> = [
  { icon: 'i-lucide-heading-1', title: 'Heading 1', action: { kind: 'line', prefix: '# ' } },
  { icon: 'i-lucide-heading-2', title: 'Heading 2', action: { kind: 'line', prefix: '## ' } },
  { icon: 'i-lucide-heading-3', title: 'Heading 3', action: { kind: 'line', prefix: '### ' } },
  'sep',
  { icon: 'i-lucide-bold', title: 'Bold', action: { kind: 'wrap', before: '**', after: '**' } },
  { icon: 'i-lucide-italic', title: 'Italic', action: { kind: 'wrap', before: '_', after: '_' } },
  { icon: 'i-lucide-strikethrough', title: 'Strikethrough', action: { kind: 'wrap', before: '~~', after: '~~' } },
  { icon: 'i-lucide-code', title: 'Inline code', action: { kind: 'wrap', before: '`', after: '`' } },
  { icon: 'i-lucide-square-code', title: 'Code block', action: { kind: 'wrap', before: '\n```\n', after: '\n```\n' } },
  'sep',
  { icon: 'i-lucide-list', title: 'Bullet list', action: { kind: 'line', prefix: '- ' } },
  { icon: 'i-lucide-list-ordered', title: 'Numbered list', action: { kind: 'line', prefix: '1. ' } },
  { icon: 'i-lucide-text-quote', title: 'Quote', action: { kind: 'line', prefix: '> ' } },
  'sep',
  { icon: 'i-lucide-link', title: 'Link', action: { kind: 'wrap', before: '[', after: '](https://)' } },
  { icon: 'i-lucide-image', title: 'Image', action: { kind: 'wrap', before: '![', after: '](https://)' } }
]

const rendered = computed(() => {
  const html = marked.parse(props.modelValue || '', { breaks: true, async: false }) as string
  // Текст пишут админы, но санитайзер всё равно нужен: новость уезжает и в
  // лаунчер, и на сайт, а один скомпрометированный админ иначе получает XSS.
  return DOMPurify.sanitize(html)
})

function apply(action: Action) {
  const el = area.value
  if (!el) return
  const text = props.modelValue || ''
  const { selectionStart: start, selectionEnd: end } = el

  let next: string
  let caret: number

  if (action.kind === 'wrap') {
    const selected = text.slice(start, end)
    next = text.slice(0, start) + action.before + selected + action.after + text.slice(end)
    caret = start + action.before.length + selected.length
  } else {
    // Префикс ставится на каждую строку выделения, а не только на первую.
    const lineStart = text.lastIndexOf('\n', start - 1) + 1
    const lineEnd = end + (text.indexOf('\n', end) === -1 ? 0 : 0)
    const block = text.slice(lineStart, Math.max(lineEnd, start))
    const prefixed = block
      .split('\n')
      .map(l => (l.startsWith(action.prefix) ? l.slice(action.prefix.length) : action.prefix + l))
      .join('\n')
    next = text.slice(0, lineStart) + prefixed + text.slice(Math.max(lineEnd, start))
    caret = lineStart + prefixed.length
  }

  emit('update:modelValue', next)
  nextTick(() => {
    el.focus()
    el.setSelectionRange(caret, caret)
  })
}

/** Ctrl/Cmd+B и Ctrl/Cmd+I — то, что руки набирают сами. */
function onKeydown(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return
  const map: Record<string, Action> = {
    b: { kind: 'wrap', before: '**', after: '**' },
    i: { kind: 'wrap', before: '_', after: '_' }
  }
  const action = map[event.key.toLowerCase()]
  if (!action) return
  event.preventDefault()
  apply(action)
}
</script>

<template>
  <div class="grid gap-2">
    <div
      class="flex flex-wrap items-center gap-1 rounded-t-[var(--noro-r-sm)] border border-b-0 border-[var(--noro-border)] bg-[var(--noro-panel-2)] px-2 py-2"
    >
      <template v-for="(tool, i) in TOOLS" :key="i">
        <div v-if="tool === 'sep'" class="mx-1 h-5 w-px bg-[var(--noro-border)]" />
        <button
          v-else
          type="button"
          class="grid size-8 place-items-center rounded-[var(--noro-r-sm)] text-[var(--noro-muted)] transition-colors duration-100 hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)] disabled:opacity-40"
          :title="tool.title"
          :aria-label="tool.title"
          :disabled="preview"
          @click="apply(tool.action)"
        >
          <UIcon :name="tool.icon" class="size-4" />
        </button>
      </template>

      <!-- Переключатель, а не кнопка: это два состояния одного редактора,
           а кнопка читалась как действие «показать превью» и не давала
           понять, включено ли оно сейчас. -->
      <label class="ml-auto flex cursor-pointer select-none items-center gap-2">
        <span class="noro-label">Preview</span>
        <button
          type="button"
          role="switch"
          :aria-checked="preview"
          class="relative h-5 w-9 shrink-0 rounded-full border transition-colors duration-100"
          :class="preview
            ? 'border-[var(--noro-cream)] bg-[var(--noro-cream)]'
            : 'border-[var(--noro-border)] bg-[var(--noro-input)]'"
          @click="preview = !preview"
        >
          <span
            class="absolute top-[2px] size-3.5 rounded-full transition-all duration-100"
            :class="preview
              ? 'left-[18px] bg-[var(--noro-on-cream)]'
              : 'left-[2px] bg-[var(--noro-muted)]'"
          />
        </button>
      </label>
    </div>

    <div
      v-if="preview"
      class="noro-markdown -mt-2 min-h-[240px] rounded-b-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] p-4"
      v-html="rendered"
    />
    <textarea
      v-else
      ref="area"
      :value="modelValue"
      :rows="rows || 14"
      spellcheck="false"
      class="noro-input -mt-2 w-full rounded-t-none font-mono text-sm leading-6"
      placeholder="Write in Markdown. Select text and use the toolbar, or press Cmd/Ctrl+B."
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      @keydown="onKeydown"
    />
  </div>
</template>
