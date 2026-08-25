<script setup lang="ts">
/**
 * Год игровой активности клетками: колонка — неделя, строка — день недели.
 *
 * Мастер отдаёт только дни, когда играли, — календарь строится здесь. Без
 * пустых клеток «один сыгранный день» выглядел одиноким квадратом в пустой
 * строке, и понять по нему было нечего.
 *
 * Шкала абсолютная, от нуля до суток. Относительная — от самого длинного дня
 * за год — красила получасовую сессию как максимум, если больше в тот год не
 * играли: карта показывала не «сколько играл», а «когда играл чаще обычного».
 */
interface DayActivity {
  day: string
  minutes: number
}

const auth = useAuth()
const { t, locale } = useT()

const days = ref<DayActivity[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    days.value = await auth.request<DayActivity[]>('/api/me/activity-heatmap')
  } catch (e) {
    console.error('Failed to load activity heatmap', e)
  } finally {
    loading.value = false
  }
}

const byDay = computed(() => new Map(days.value.map(d => [d.day, d.minutes])))

function iso(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

const weeks = computed(() => {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const start = new Date(today)
  start.setDate(start.getDate() - 364)
  // getDay(): 0 — воскресенье. Сдвигаем к понедельнику, иначе строки съезжают.
  start.setDate(start.getDate() - ((start.getDay() + 6) % 7))

  const out: { key: string, date: Date, minutes: number, future: boolean }[][] = []
  const cursor = new Date(start)
  while (cursor <= today) {
    const week: typeof out[number] = []
    for (let i = 0; i < 7; i++) {
      const key = iso(cursor)
      week.push({ key, date: new Date(cursor), minutes: byDay.value.get(key) ?? 0, future: cursor > today })
      cursor.setDate(cursor.getDate() + 1)
    }
    out.push(week)
  }
  return out
})

/** Подпись месяца — над неделей, в которой он начался. */
const monthLabels = computed(() => {
  const format = new Intl.DateTimeFormat(locale.value, { month: 'short' })
  let previous = -1
  return weeks.value.map((week) => {
    const month = week[0]!.date.getMonth()
    if (month === previous) return ''
    previous = month
    return format.format(week[0]!.date)
  })
})

/**
 * Ступени в часах: сутки — потолок, который не переставляется.
 *
 * Границы неравномерные намеренно: разница между «зашёл на полчаса» и «играл
 * два часа» важнее, чем между восемью часами и десятью.
 */
const STEPS = [1, 3, 6, 12]

function level(minutes: number) {
  if (minutes <= 0) return 0
  const hours = minutes / 60
  return STEPS.findIndex(step => hours < step) + 1 || STEPS.length + 1
}

/**
 * Ступени заливкой, без рамок.
 *
 * Рамка на клетке в двенадцать пикселей — это не контур, а шум: сетка пустых
 * дней превращалась в забор из квадратиков и забивала собой те дни, ради
 * которых карту и открывают.
 */
const LEVELS = [
  'bg-[color-mix(in_srgb,var(--noro-border)_45%,transparent)]',
  'bg-[color-mix(in_srgb,var(--noro-green)_25%,var(--noro-input))]',
  'bg-[color-mix(in_srgb,var(--noro-green)_50%,var(--noro-input))]',
  'bg-[color-mix(in_srgb,var(--noro-green)_75%,var(--noro-input))]',
  'bg-[var(--noro-green)]',
  'bg-[var(--noro-cream)]',
]

/** Часы человеческой строкой: «2 ч 30 мин», «45 мин». */
function span(minutes: number) {
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  if (!hours) return t('cabinet-activity-minutes', { count: rest })
  if (!rest) return t('cabinet-activity-hours', { count: hours })
  return `${t('cabinet-activity-hours', { count: hours })} ${t('cabinet-activity-minutes', { count: rest })}`
}

/**
 * Подсказка своя, а не `title`: нативная всплывает через полсекунды и уходит
 * сама, а по карте водят курсором быстро — родная просто не успевает.
 */
const hovered = ref<{ text: string, x: number, y: number } | null>(null)

function show(event: MouseEvent, day: { key: string, minutes: number }) {
  const cell = event.currentTarget as HTMLElement
  const box = cell.getBoundingClientRect()
  const host = cell.closest('[data-heatmap]')!.getBoundingClientRect()
  const date = new Intl.DateTimeFormat(locale.value, { dateStyle: 'long' }).format(new Date(day.key))
  const text = `${date} — ${day.minutes ? span(day.minutes) : t('cabinet-activity-none')}`

  // Шрифт моноширинный, поэтому ширину подсказки можно посчитать, не рисуя её:
  // без этого у крайних клеток она наполовину уезжала за край карточки.
  const half = (text.length * 6 + 16) / 2
  const centre = box.left - host.left + box.width / 2
  hovered.value = {
    text,
    x: Math.min(Math.max(centre, half), host.width - half),
    y: box.top - host.top,
  }
}

/** Итог года: он отвечает на вопрос быстрее, чем разглядывание клеток. */
const total = computed(() => days.value.reduce((sum, d) => sum + d.minutes, 0))
const best = computed(() => Math.max(0, ...days.value.map(d => d.minutes)))

onMounted(() => load())
</script>

<template>
  <NoroCard :title="t('cabinet-activity-title')" icon="i-lucide-calendar-days">
    <template #actions>
      <span class="text-xs text-[var(--noro-muted)]">
        {{ t('cabinet-activity-days', { count: 365 }) }}
      </span>
    </template>

    <div v-if="days.length" class="space-y-3">
      <div class="flex flex-wrap gap-x-6 gap-y-1 text-xs">
        <span class="text-[var(--noro-muted)]">
          {{ t('cabinet-activity-total') }}
          <b class="ml-1 text-[var(--noro-text)]">{{ span(total) }}</b>
        </span>
        <span class="text-[var(--noro-muted)]">
          {{ t('cabinet-activity-best') }}
          <b class="ml-1 text-[var(--noro-text)]">{{ span(best) }}</b>
        </span>
      </div>

      <!-- Обёртка `relative` не прокручивается сама: подсказку, лежащую внутри
           скроллера, обрезало бы `overflow`, а у верхнего ряда — ещё и краем
           карточки. Отступ сверху и есть место под неё. -->
      <div data-heatmap class="relative pt-8">
        <div
          class="pointer-events-none absolute inset-x-0 top-2 grid gap-1"
          :style="{ gridTemplateColumns: `repeat(${weeks.length}, minmax(0, 1fr))` }"
        >
          <span v-for="(label, index) in monthLabels" :key="index" class="relative">
            <span
              v-if="label"
              class="absolute left-0 top-0 whitespace-nowrap text-[10px] leading-none text-[var(--noro-muted)]"
            >{{ label }}</span>
          </span>
        </div>

        <!-- Год растягивается на всю ширину карточки: колонок ровно столько,
             сколько недель, а клетка квадратная от доли ширины. Фиксированные
             12 пикселей оставляли справа пустую четверть. -->
        <div
          class="grid grid-flow-col grid-rows-7 gap-1"
          :style="{ gridTemplateColumns: `repeat(${weeks.length}, minmax(0, 1fr))` }"
          @mouseleave="hovered = null"
        >
          <div
            v-for="day in weeks.flat()"
            :key="day.key"
            class="aspect-square w-full rounded-[2px] transition-shadow hover:shadow-[inset_0_0_0_1.5px_var(--noro-cream)]"
            :class="[LEVELS[level(day.minutes)], day.future ? 'invisible' : '']"
            @mouseenter="show($event, day)"
          />
        </div>

        <div
          v-if="hovered"
          class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full whitespace-nowrap rounded border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-2 py-1 text-[10px] text-[var(--noro-text)] shadow-lg"
          :style="{ left: `${hovered.x}px`, top: `${hovered.y - 4}px` }"
        >{{ hovered.text }}</div>
      </div>

      <!-- Легенда в часах, а не «меньше/больше»: шкала абсолютная, и по ней
           можно сказать, сколько именно наиграно, не наводя курсор. -->
      <div class="flex items-center gap-1 text-[10px] text-[var(--noro-muted)]">
        <span class="size-3 rounded-[2px]" :class="LEVELS[0]" />
        <span class="mr-2">0</span>
        <template v-for="(step, i) in STEPS" :key="step">
          <span class="size-3 rounded-[2px]" :class="LEVELS[i + 1]" />
          <span class="mr-2">&lt;{{ step }}{{ t('cabinet-activity-hour-short') }}</span>
        </template>
        <span class="size-3 rounded-[2px]" :class="LEVELS[5]" />
        <span>12{{ t('cabinet-activity-hour-short') }}+</span>
      </div>
    </div>

    <EmptyState
      v-else-if="!loading"
      icon="i-lucide-calendar"
      :title="t('cabinet-activity-empty-title')"
      :text="t('cabinet-activity-empty-text')"
    />
  </NoroCard>
</template>
