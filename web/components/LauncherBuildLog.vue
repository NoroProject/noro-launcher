<script setup lang="ts">
/**
 * Лог сборки лаунчера.
 *
 * Раньше здесь надо было откуда-то взять job_id, вставить его в поле и нажать
 * кнопку, а лог показывался как JSON.stringify — одной строкой с \n внутри и
 * горизонтальной прокруткой. Теперь список задач приходит с мастера, лог живой
 * и переносится по строкам.
 */
interface BuildJob {
  id: string
  tag: string
  status: string
  created_at: string
}

const props = defineProps<{ jobId?: string | null }>()
const open = defineModel<boolean>({ required: true })

const auth = useAuth()
const jobs = ref<BuildJob[]>([])
const selected = ref<string>('')
const log = ref('')
const status = ref('')
const pending = ref(false)
const body = ref<HTMLElement | null>(null)
let timer: ReturnType<typeof setInterval> | null = null

/** Пока сборка идёт, лог дописывается — есть смысл перечитывать. */
const live = computed(() => ['pending', 'building', 'downloading'].includes(status.value))

async function loadJobs() {
  try {
    jobs.value = await auth.request<BuildJob[]>('/api/admin/launcher/builds')
    if (!selected.value) selected.value = props.jobId || jobs.value[0]?.id || ''
  } catch { /* список не критичен: id можно выбрать позже */ }
}

async function loadLog() {
  if (!selected.value) return
  pending.value = true
  try {
    const data = await auth.request<{ status: string, log: string }>(
      `/api/admin/launcher/build/${selected.value}/log`
    )
    status.value = data.status
    log.value = data.log || '(пусто)'
    // Хвост важнее начала: интересно последнее, что произошло.
    await nextTick()
    if (body.value) body.value.scrollTop = body.value.scrollHeight
  } catch (e) {
    log.value = humanError(e)
    status.value = 'failed'
  } finally {
    pending.value = false
  }
}

watch(open, async (isOpen) => {
  if (!isOpen) return
  selected.value = props.jobId || selected.value
  await loadJobs()
  await loadLog()
})

watch(selected, () => void loadLog())

// Опрос только на открытой модалке и только пока сборка не завершилась.
watchEffect(() => {
  if (timer) clearInterval(timer)
  timer = open.value && live.value ? setInterval(loadLog, 2000) : null
})

onBeforeUnmount(() => timer && clearInterval(timer))

function badge(value: string) {
  if (value === 'done') return 'success'
  if (value === 'failed') return 'error'
  return 'info'
}

function label(job: BuildJob) {
  return `${job.tag} · ${job.status} · ${new Date(job.created_at).toLocaleString()}`
}
</script>

<template>
  <AtomModal v-model="open" title="BUILD LOG" subtitle="Inspect launcher builder output" wide>
    <div class="flex flex-wrap items-center gap-3">
      <NoroSelect v-model="selected" class="min-w-0 flex-1">
        <option v-if="!jobs.length" value="">No builds yet</option>
        <option v-for="job in jobs" :key="job.id" :value="job.id">{{ label(job) }}</option>
      </NoroSelect>
      <UBadge v-if="status" :color="badge(status)" variant="subtle">{{ status }}</UBadge>
      <AtomButton
        variant="secondary"
        size="sm"
        icon="i-lucide-refresh-cw"
        :loading="pending"
        @click="loadLog"
      />
    </div>

    <p v-if="live" class="mt-2 text-xs text-[var(--noro-muted)]">
      Build is running — refreshing every 2s.
    </p>

    <pre
      v-if="log"
      ref="body"
      class="mt-3 max-h-96 overflow-y-auto whitespace-pre-wrap break-words rounded-[var(--noro-r-sm)]
             bg-[var(--noro-input)] p-3 text-xs leading-5 text-[var(--noro-text)]"
    >{{ log }}</pre>
  </AtomModal>
</template>
