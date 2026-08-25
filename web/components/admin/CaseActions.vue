<script setup lang="ts">
/**
 * Панель разбора: замок, запросы к игре и закрытие с вердиктом.
 *
 * Запросы отвечают «попросили», а не результатом: ответ приезжает кадром от
 * агента и появляется в ленте сам. Поэтому они и собраны отдельной группой —
 * нажатие здесь не меняет карточку мгновенно, и ждать этого не надо.
 */
import type { CaseRow } from '~/types/cases'

const props = defineProps<{ item: CaseRow }>()
const emit = defineEmits<{ act: [path: string, method?: 'POST' | 'PUT', body?: Record<string, unknown>] }>()
const { t } = useT()

const note = ref('')
const resolution = ref('')
const verdict = ref<'confirmed' | 'rejected' | 'insufficient'>('confirmed')
/** Что сейчас летит на сервер: плитка крутится, повторное нажатие гасится. */
const busy = ref<string | null>(null)

const active = computed(() => ['open', 'in_review'].includes(props.item.status))
const mine = computed(() => props.item.status === 'in_review')

function ask(path: string) {
  busy.value = path
  emit('act', path)
  // Ответ приходит не сюда, а кадром в ленту: держать плитку в ожидании
  // дольше секунды значило бы врать про связь с игрой.
  setTimeout(() => (busy.value = null), 1000)
}

function sendNote() {
  if (!note.value.trim()) return
  emit('act', 'notes', 'POST', { text: note.value.trim() })
  note.value = ''
}

function close() {
  emit('act', 'resolve', 'PUT', {
    verdict: verdict.value,
    resolution: resolution.value.trim(),
    rule_code: props.item.rule_code,
  })
}
</script>

<template>
  <div class="noro-panel grid min-w-0 grid-cols-[minmax(0,1fr)] gap-5 p-4">
    <AdminCaseActionTile
      v-if="active"
      :icon="mine ? 'i-lucide-undo-2' : 'i-lucide-hand'"
      :label="mine ? t('admin-case-release') : t('admin-case-claim')"
      :hint="mine ? t('admin-case-release-hint') : t('admin-case-claim-hint')"
      @click="emit('act', mine ? 'release' : 'claim')"
    />

    <section class="grid gap-2">
      <span class="noro-label">{{ t('admin-case-probes') }}</span>
      <div class="grid gap-2">
        <AdminCaseActionTile
          icon="i-lucide-message-square"
          :label="t('admin-case-probe-chat')"
          :hint="t('admin-case-probe-chat-hint')"
          :busy="busy === 'chat-request'"
          @click="ask('chat-request')"
        />
        <AdminCaseActionTile
          icon="i-lucide-backpack"
          :label="t('admin-case-probe-inventory')"
          :hint="t('admin-case-probe-inventory-hint')"
          :busy="busy === 'inventory-request'"
          @click="ask('inventory-request')"
        />
        <AdminCaseActionTile
          icon="i-lucide-shield-check"
          :label="t('admin-case-probe-client')"
          :hint="t('admin-case-probe-client-hint')"
          :busy="busy === 'client-check'"
          @click="ask('client-check')"
        />
      </div>
    </section>

    <label class="grid gap-2">
      <span class="noro-label">{{ t('admin-case-note') }}</span>
      <textarea v-model="note" rows="2" class="noro-input w-full" :placeholder="t('admin-case-note-hint')" />
      <AtomButton size="sm" icon="i-lucide-plus" variant="secondary" :disabled="!note.trim()" @click="sendNote">
        {{ t('admin-case-note-add') }}
      </AtomButton>
    </label>

    <section v-if="active" class="grid gap-3 border-t border-[var(--noro-border)] pt-4">
      <span class="noro-label">{{ t('admin-case-close') }}</span>
      <AdminCaseVerdictPicker v-model="verdict" />
      <input v-model="resolution" class="noro-input w-full" :placeholder="t('admin-case-resolution-hint')">
      <AtomButton icon="i-lucide-scale" variant="primary" @click="close">
        {{ t('admin-case-close-do') }}
      </AtomButton>
    </section>
  </div>
</template>
