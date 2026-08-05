<script setup lang="ts">
withDefaults(defineProps<{
  /** `compact` — вариант для кабинета, без заголовка и пояснения. */
  compact?: boolean
}>(), {})

const { downloads, pending, primary, others, label, href, size } = useLauncherDownloads()
const showAll = ref(false)
</script>

<template>
  <section :class="compact ? '' : 'noro-panel p-6'">
    <div v-if="!compact" class="mb-4">
      <h2 class="noro-pixel text-2xl text-[var(--noro-cream)]">GET THE LAUNCHER</h2>
      <p class="mt-2 text-sm text-[var(--noro-muted)]">
        Sign in with Discord, pick a server, and the launcher syncs the rest.
      </p>
    </div>

    <div v-if="pending" class="text-sm text-[var(--noro-muted)]">Loading builds…</div>

    <!-- Пока релиза нет, кнопка вела бы в никуда. Честнее сказать прямо. -->
    <div v-else-if="!downloads.length" class="text-sm text-[var(--noro-muted)]">
      No launcher build published yet.
    </div>

    <template v-else>
      <!-- Платформы может не быть: при рендере на сервере она неизвестна. Тогда
           вместо кнопки сразу раскрыт полный список, чтобы никому не досталась
           сборка под чужую систему. -->
      <div v-if="primary" class="flex flex-wrap items-center gap-3">
        <AtomButton
          variant="primary"
          size="lg"
          icon="i-lucide-download"
          :href="href(primary)"
          download
        >
          Download for {{ label(primary.platform) }}
        </AtomButton>
        <span class="text-xs text-[var(--noro-muted)]">
          v{{ primary.version }} · {{ size(primary.size) }}
        </span>
      </div>

      <div v-if="others.length" :class="primary ? 'mt-4' : ''">
        <AtomButton
          v-if="primary"
          variant="ghost"
          size="sm"
          :icon="showAll ? 'i-lucide-chevron-up' : 'i-lucide-chevron-down'"
          @click="showAll = !showAll"
        >
          Other platforms
        </AtomButton>

        <div v-if="showAll || !primary" :class="primary ? 'mt-3 grid gap-2' : 'grid gap-2'">
          <a
            v-for="item in others"
            :key="item.platform"
            :href="href(item)"
            download
            class="flex items-center justify-between rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-2 text-sm transition-colors duration-100 hover:border-[var(--noro-muted)]"
          >
            <span class="text-[var(--noro-text)]">{{ label(item.platform) }}</span>
            <span class="text-xs text-[var(--noro-muted)]">{{ size(item.size) }}</span>
          </a>
        </div>
      </div>

      <p v-if="!compact" class="mt-4 text-xs text-[var(--noro-muted)]">
        Every build is signed — the launcher checks the signature before it runs.
      </p>
    </template>
  </section>
</template>
