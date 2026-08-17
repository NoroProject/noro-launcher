<script setup lang="ts">
/** Карточка сервера: баннер, иконка внахлёст, онлайн и ссылка на правила. */
import type { PublicServer } from '~/types/public'

const props = defineProps<{ server: PublicServer }>()
const { t } = useT()

const fill = computed(() => {
  const { online, max_online: max } = props.server
  return max > 0 ? Math.min(100, Math.round((online / max) * 100)) : 0
})
</script>

<template>
  <article
    class="noro-panel group relative flex flex-col overflow-hidden rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel)] transition-all duration-300 hover:-translate-y-1 hover:border-[var(--noro-cream)] hover:shadow-[0_12px_32px_rgba(0,0,0,0.4)]"
  >
    <!-- Top Banner Area (h-36) -->
    <div class="relative h-36 w-full overflow-hidden bg-[var(--noro-bg-deep)]">
      <!-- Background Image -->
      <img
        v-if="server.background_url"
        :src="server.background_url"
        class="h-full w-full object-cover opacity-85 transition-transform duration-700 group-hover:scale-105"
        alt=""
      >
      <!-- Fallback Ambient Gradient Pattern -->
      <div v-else class="relative h-full w-full bg-gradient-to-br from-[var(--noro-bg-deep)] via-[var(--noro-panel)] to-[var(--noro-sidebar)]">
        <div class="absolute inset-0 opacity-15 [background-image:radial-gradient(#ebt176_1px,transparent_1px)] [background-size:16px_16px]" />
      </div>

      <!-- Gradient Overlay for Text Contrast -->
      <div class="absolute inset-0 bg-gradient-to-t from-[var(--noro-panel)] via-black/20 to-black/40" />

      <!-- Status Badge Top Right -->
      <div class="absolute right-3 top-3 z-10">
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-black/65 px-3 py-1 text-[11px] font-black uppercase tracking-wider backdrop-blur-md"
          :class="server.live ? 'text-[var(--noro-green)]' : 'text-[var(--noro-muted)]'"
        >
          <span
            class="size-2 rounded-full"
            :class="server.live ? 'bg-[var(--noro-green)] shadow-[0_0_8px_var(--noro-green)]' : 'bg-[var(--noro-muted)]'"
          />
          {{ server.live ? t('web-server-online') : t('web-server-offline') }}
        </span>
      </div>
    </div>

    <!-- Content Body with Overlapping Avatar -->
    <div class="relative flex flex-1 flex-col px-5 pb-5 pt-0">
      <!-- Avatar + Version Badges Row -->
      <div class="-mt-8 mb-3 flex items-end justify-between gap-3">
        <!-- Overlapping Server Icon -->
        <div class="relative z-10 shrink-0">
          <img
            v-if="server.icon_url"
            :src="server.icon_url"
            class="size-16 rounded-xl border-2 border-[var(--noro-panel)] bg-[var(--noro-bg-deep)] object-cover shadow-xl transition-transform duration-300 group-hover:scale-105"
            alt=""
          >
          <div
            v-else
            class="grid size-16 place-items-center rounded-xl border-2 border-[var(--noro-panel)] bg-[var(--noro-input)] text-[var(--noro-muted)] shadow-xl"
          >
            <UIcon name="i-lucide-box" class="size-8" />
          </div>
        </div>

        <!-- Badges (mc_version & modloader) -->
        <div class="flex flex-wrap items-center gap-1.5 pb-0.5">
          <span class="rounded-md border border-[var(--noro-blue)]/30 bg-[var(--noro-blue)]/10 px-2.5 py-1 font-mono text-xs font-black text-[var(--noro-blue)]">
            {{ server.mc_version }}
          </span>
          <span class="rounded-md border border-[var(--noro-border)] bg-[var(--noro-input)] px-2.5 py-1 text-xs font-black uppercase tracking-wider text-[var(--noro-muted)]">
            {{ server.modloader }}
          </span>
        </div>
      </div>

      <!-- Server Title & Description -->
      <div class="mb-4">
        <h3 class="noro-pixel truncate text-lg uppercase text-[var(--noro-cream)] transition-colors group-hover:text-white">
          {{ server.name }}
        </h3>
        <p v-if="server.description" class="mt-2 line-clamp-2 text-xs font-medium leading-5 text-[var(--noro-muted)]">
          {{ server.description }}
        </p>
      </div>

      <!-- Online Players Bar -->
      <div class="mt-auto pt-2">
        <div class="mb-1.5 flex items-baseline justify-between text-xs">
          <span class="noro-label">{{ t('web-server-players') }}</span>
          <span class="font-mono text-xs font-bold text-[var(--noro-text)]">
            <span class="font-black text-[var(--noro-cream)]">{{ server.online }}</span>
            <span class="text-[var(--noro-muted)]"> / {{ server.max_online || '—' }}</span>
          </span>
        </div>
        <div class="h-2 w-full overflow-hidden rounded-full bg-[var(--noro-input)]">
          <div
            class="h-full rounded-full bg-gradient-to-r from-[var(--noro-blue)] to-[var(--noro-green)] transition-all duration-500"
            :style="{ width: `${fill}%` }"
          />
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="mt-4 flex items-center justify-between border-t border-[var(--noro-border-soft)] pt-3">
        <AtomButton
          variant="ghost"
          size="sm"
          icon="i-lucide-book-open"
          :to="`/rules?server=${server.id}`"
        >
          {{ t('web-server-rules') }}
        </AtomButton>
      </div>
    </div>
  </article>
</template>
