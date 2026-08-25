<script setup lang="ts">
/**
 * Срез чата, приложенный к делу.
 *
 * Канал показан всегда: «он мне угрожал» в общем чате и в личке — разные
 * разговоры, и по строке без пометки их не различить.
 */
import type { CaseMessage } from '~/types/cases'

const props = defineProps<{ messages: CaseMessage[], allowed: boolean }>()
const { t } = useT()

const channel = ref<'all' | CaseMessage['channel']>('all')
const CHANNELS = ['all', 'public', 'local', 'private', 'command'] as const

const CHANNEL_TONE: Record<string, string> = {
  public: 'text-[var(--noro-text)]',
  local: 'text-[var(--noro-text)]',
  private: 'text-amber-400',
  command: 'text-[var(--noro-blue)]',
}

const shown = computed(() =>
  channel.value === 'all'
    ? props.messages
    : props.messages.filter(m => m.channel === channel.value),
)
</script>

<template>
  <div class="grid gap-3">
    <NoroNote v-if="!allowed" icon="i-lucide-lock">
      {{ t('admin-case-chat-forbidden') }}
    </NoroNote>

    <template v-else>
      <div class="flex flex-wrap gap-2">
        <AtomButton
          v-for="option in CHANNELS"
          :key="option"
          size="sm"
          :variant="channel === option ? 'primary' : 'dark'"
          @click="channel = option"
        >
          {{ t(`admin-case-channel-${option}`) }}
        </AtomButton>
      </div>

      <!-- Время колонкой и моноширинно: реплики читают как расшифровку, и
           выравнивание слева важнее пары сэкономленных пикселей. -->
      <div v-if="shown.length" class="grid max-h-[28rem] gap-1 overflow-y-auto pr-1">
        <div
          v-for="message in shown"
          :key="message.id"
          class="grid grid-cols-[3rem_minmax(0,1fr)] items-baseline gap-x-3 text-sm leading-6"
        >
          <time class="text-right font-mono text-xs text-[var(--noro-muted)]">{{ clockTime(message.at) }}</time>
          <p class="min-w-0 break-words">
            <span class="font-bold text-white">{{ message.sender_name }}</span>
            <span
              v-if="message.channel !== 'public'"
              class="ml-2 text-[10px] uppercase tracking-wide text-[var(--noro-muted)]"
            >{{ t(`admin-case-channel-${message.channel}`) }}</span>
            <span class="ml-2" :class="CHANNEL_TONE[message.channel]">{{ message.content }}</span>
          </p>
        </div>
      </div>
      <p v-else class="text-sm text-[var(--noro-muted)]">{{ t('admin-case-chat-empty') }}</p>
    </template>
  </div>
</template>
