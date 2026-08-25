<script setup lang="ts">
/**
 * Лента разбора: что делали с делом, когда и откуда.
 *
 * Время слева колонкой и моноширинно — по нему видно темп разбора: три жалобы
 * за двадцать минут читаются иначе, чем три за неделю. Источник подписан
 * словом, а не значком: спор «я этого не делал» решается тем, видно ли, что
 * телепорт пришёл из игры, а вердикт — с сайта.
 */
import type { CaseEvent } from '~/types/cases'

defineProps<{ events: CaseEvent[] }>()
const { t } = useT()

const ICONS: Record<string, string> = {
  report_added: 'i-lucide-flag',
  claimed: 'i-lucide-hand',
  released: 'i-lucide-undo-2',
  teleport: 'i-lucide-move-3d',
  freeze: 'i-lucide-snowflake',
  watch_start: 'i-lucide-eye',
  watch_stop: 'i-lucide-eye-off',
  inventory_snapshot: 'i-lucide-backpack',
  chat_slice: 'i-lucide-message-square',
  client_check: 'i-lucide-shield-check',
  screenshot: 'i-lucide-camera',
  note: 'i-lucide-sticky-note',
  punishment: 'i-lucide-gavel',
  verdict: 'i-lucide-scale',
}

/** Вердикт и наказание — итог разбора, их видно с другого конца комнаты. */
const LOUD = new Set(['punishment', 'verdict', 'report_added'])

function detail(event: CaseEvent) {
  const p = (event.payload || {}) as Record<string, unknown>
  if (event.kind === 'note') return String(p.text ?? '')
  if (event.kind === 'punishment') return `${p.kind ?? ''} · ${p.reason ?? ''}`
  if (event.kind === 'verdict') return `${p.verdict ?? ''} · ${p.resolution ?? ''}`
  if (event.kind === 'report_added') return String(p.reason ?? '')
  if (event.kind === 'screenshot') return String(p.note ?? '')
  if (event.kind === 'chat_slice') return t('admin-case-chat-saved', { count: Number(p.messages ?? 0) })
  if (event.kind === 'teleport') return String(p.to ?? '')
  if (event.kind === 'inventory_snapshot') {
    const items = Array.isArray(p.items) ? (p.items as string[]) : []
    return items.join(', ')
  }
  return ''
}

/** Кадр показывается сразу: ссылка «файл» в ленте разбора никем не открывается. */
function shot(event: CaseEvent): string {
  if (event.kind !== 'screenshot') return ''
  return String((event.payload as Record<string, unknown> | null)?.file_url ?? '')
}
</script>

<template>
  <!-- Своя прокрутка, как у среза чата: лента растёт на каждое действие, и у
       разбора на полсотни событий она уводила форму наказания и кнопки далеко
       вниз — до них приходилось листать всю страницу. Высота та же, что у чата:
       две колонки рядом должны кончаться на одной линии. -->
  <ol class="noro-scroll relative grid max-h-[28rem] gap-0 overflow-y-auto pr-1">
    <li
      v-for="event in events"
      :key="event.id"
      class="grid grid-cols-[3.5rem_1.5rem_minmax(0,1fr)] items-start gap-x-3 py-2"
    >
      <time class="pt-0.5 text-right font-mono text-xs text-[var(--noro-muted)]">
        {{ clockTime(event.at) }}
      </time>

      <!-- Линия проходит через значки: без неё события расползаются на строки
           разной высоты и перестают читаться как одна история. -->
      <span class="relative flex justify-center self-stretch">
        <span class="absolute top-0 bottom-[-0.5rem] w-px bg-[var(--noro-border)]" />
        <UIcon
          :name="ICONS[event.kind] || 'i-lucide-dot'"
          class="relative z-10 size-4 bg-[var(--noro-panel)]"
          :class="LOUD.has(event.kind) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
        />
      </span>

      <div class="min-w-0">
        <div class="flex flex-wrap items-baseline gap-x-2">
          <span
            class="text-sm font-bold"
            :class="LOUD.has(event.kind) ? 'text-white' : 'text-[var(--noro-text)]'"
          >{{ t(`admin-case-event-${event.kind}`) }}</span>
          <span v-if="event.actor_label" class="text-xs text-[var(--noro-muted)]">
            {{ event.actor_label }}
          </span>
          <span class="text-[10px] uppercase tracking-wide text-[var(--noro-muted)]">
            {{ t(`admin-case-source-${event.source}`) }}
          </span>
        </div>
        <p v-if="detail(event)" class="mt-0.5 break-words text-sm leading-6 text-[var(--noro-text)]">
          {{ detail(event) }}
        </p>
        <a
          v-if="shot(event)"
          :href="shot(event)"
          target="_blank"
          rel="noopener"
          class="mt-1 block w-40 border border-[var(--noro-border)]"
        >
          <img :src="shot(event)" alt="" class="block w-full">
        </a>
      </div>
    </li>
  </ol>
</template>
