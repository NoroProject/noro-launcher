<script setup lang="ts">
/** Подвал публичного сайта. */
const { t } = useT()
// Строкой, а не числом: Fluent форматирует числа по локали, и в русском год
// превратился бы в «2 026».
const year = String(new Date().getFullYear())

const links = computed(() => [
  { label: t('web-nav-servers'), to: '/servers' },
  { label: t('web-nav-rules'), to: '/rules' },
  { label: t('web-nav-cabinet'), to: '/cabinet' },
])
</script>

<template>
  <footer class="mt-16 border-t border-[var(--noro-border)] bg-[var(--noro-bg-deep)]">
    <div class="mx-auto flex max-w-7xl flex-col gap-6 px-5 py-10 md:flex-row md:items-center md:justify-between">
      <NuxtLink :to="link.home()" class="flex items-center gap-3 transition hover:opacity-80">
        <img src="/icon.png" class="size-10" alt="">
        <div>
          <div class="noro-pixel text-base uppercase text-[var(--noro-cream)]">Noro</div>
          <div class="text-xs font-bold uppercase tracking-wider text-[var(--noro-muted)]">
            {{ t('web-footer-tagline') }}
          </div>
        </div>
      </NuxtLink>

      <nav class="flex flex-wrap items-center gap-4">
        <NuxtLink
          v-for="link in links"
          :key="link.to"
          :to="link.to"
          class="text-sm font-bold uppercase tracking-wider text-[var(--noro-muted)] transition hover:text-[var(--noro-text)]"
        >{{ link.label }}</NuxtLink>
      </nav>

      <p class="text-xs font-medium text-[var(--noro-muted)]">
        {{ t('web-footer-legal', { year }) }}
      </p>
    </div>
  </footer>
</template>
