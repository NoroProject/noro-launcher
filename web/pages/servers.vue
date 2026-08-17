<script setup lang="ts">
/** Серверы проекта: витрина с онлайном. Открыта без входа. */
definePageMeta({ layout: 'public' })

const { t } = useT()
const { servers, online, pending, refresh } = usePublicServers()

useHead({
  title: () => t('web-servers-meta-title'),
  meta: [{ name: 'description', content: () => t('web-servers-meta-description') }],
})
</script>

<template>
  <div class="mx-auto max-w-7xl px-5 py-10">
    <section class="flex flex-wrap items-end justify-between gap-4 border-b border-[var(--noro-border)] pb-8">
      <div>
        <h1 class="noro-pixel text-3xl uppercase text-[var(--noro-cream)] md:text-4xl">
          {{ t('web-servers-title') }}
        </h1>
        <p class="mt-3 max-w-2xl text-base font-medium leading-7 text-[var(--noro-text)]">
          {{ t('web-servers-lead') }}
        </p>
      </div>
      <div class="flex items-center gap-4">
        <div class="text-right">
          <div class="noro-label">{{ t('web-servers-online') }}</div>
          <div class="noro-pixel text-2xl text-[var(--noro-green)]">{{ online }}</div>
        </div>
        <AtomButton variant="dark" icon="i-lucide-refresh-cw" :loading="pending" @click="refresh()">
          {{ t('web-servers-refresh') }}
        </AtomButton>
      </div>
    </section>

    <div v-if="pending && !servers.length" class="noro-panel mt-8 grid place-items-center gap-2 p-12 text-[var(--noro-muted)]">
      <UIcon name="i-lucide-loader-2" class="size-6 animate-spin" />
      <span class="text-sm font-bold">{{ t('web-servers-loading') }}</span>
    </div>

    <EmptyState
      v-else-if="!servers.length"
      class="mt-8"
      icon="i-lucide-server-off"
      :title="t('web-servers-empty-title')"
      :text="t('web-servers-empty-text')"
    />

    <div v-else class="mt-8 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <PublicServerCard v-for="server in servers" :key="server.id" :server="server" />
    </div>

    <section class="mt-12">
      <LauncherDownload />
    </section>
  </div>
</template>
