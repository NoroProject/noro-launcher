<script setup lang="ts">
/** Шапка публичного сайта: одна навигация на всех страницах без входа. */
const auth = useAuth()
const route = useRoute()
const { t } = useT()
const brand = usePublicSettings()

const links = computed(() => [
  { label: t('web-nav-home'), to: '/' },
  { label: t('web-nav-servers'), to: '/servers' },
  { label: t('web-nav-rules'), to: '/rules' },
])

const open = ref(false)
const target = computed(() => (auth.loggedIn.value ? '/cabinet' : '/login'))
const isActive = (to: string) => (to === '/' ? route.path === '/' : route.path.startsWith(to))

onMounted(() => {
  if (auth.loggedIn.value && !auth.user.value) {
    auth.loadMe()
  }
})

// Меню на телефоне закрывается вместе с переходом: иначе оно остаётся поверх
// новой страницы и выглядит как зависший экран.
watch(() => route.fullPath, () => { open.value = false })
</script>

<template>
  <header class="sticky top-0 z-40 border-b border-[var(--noro-border)] bg-[var(--noro-bg-deep)]/95 backdrop-blur">
    <div class="mx-auto flex max-w-7xl items-center gap-4 px-5 py-4">
      <NuxtLink :to="link.home()" class="flex min-w-0 items-center gap-3">
        <img :src="brand.logo_url || '/icon.png'" class="size-10 shrink-0 object-contain" alt="">
        <span class="noro-pixel truncate text-lg uppercase text-[var(--noro-cream)]">{{ brand.instance_name }}</span>
      </NuxtLink>

      <nav class="ml-6 hidden items-center gap-1 md:flex">
        <NuxtLink
          v-for="link in links"
          :key="link.to"
          :to="link.to"
          class="rounded-[var(--noro-r-sm)] px-3 py-2 text-sm font-bold uppercase tracking-wider transition"
          :class="isActive(link.to)
            ? 'bg-[var(--noro-panel)] text-[var(--noro-cream)]'
            : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
        >{{ link.label }}</NuxtLink>
      </nav>

      <div class="ml-auto flex items-center gap-2">
        <LocaleSwitch />
        <NuxtLink
          :to="target"
          class="hidden h-9 items-center justify-center gap-2 rounded-[var(--noro-r-sm)] px-3.5 text-xs font-bold uppercase tracking-wider transition sm:inline-flex"
          :class="auth.loggedIn.value
            ? 'bg-[var(--noro-panel)] text-[var(--noro-cream)] hover:bg-[var(--noro-panel-2)] hover:scale-[1.02]'
            : 'noro-cta'"
        >
          <img
            v-if="auth.loggedIn.value && identityAvatar(auth.user.value)"
            :src="identityAvatar(auth.user.value)!"
            alt=""
            class="size-5 rounded object-cover"
          >
          <UIcon v-else :name="auth.loggedIn.value ? 'i-lucide-user' : 'i-lucide-log-in'" class="size-3.5" />
          <span class="max-w-[120px] truncate">
            {{ auth.loggedIn.value ? (auth.user.value?.username || t('web-nav-cabinet')) : t('web-nav-sign-in') }}
          </span>
        </NuxtLink>
        <button
          type="button"
          class="rounded-[var(--noro-r-sm)] p-2 text-[var(--noro-muted)] transition hover:bg-[var(--noro-panel)] hover:text-[var(--noro-text)] md:hidden"
          :aria-label="open ? t('web-nav-menu-close') : t('web-nav-menu-open')"
          @click="open = !open"
        >
          <UIcon :name="open ? 'i-lucide-x' : 'i-lucide-menu'" class="size-5" />
        </button>
      </div>
    </div>

    <div v-if="open" class="border-t border-[var(--noro-border)] px-5 py-3 md:hidden">
      <NuxtLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        class="block rounded-[var(--noro-r-sm)] px-3 py-2 text-sm font-bold uppercase tracking-wider"
        :class="isActive(link.to) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
      >{{ link.label }}</NuxtLink>
      <NuxtLink
        :to="target"
        class="mt-2 flex w-full items-center justify-center gap-2 rounded-[var(--noro-r-sm)] px-4 py-2 text-xs font-bold uppercase tracking-wider"
        :class="auth.loggedIn.value ? 'bg-[var(--noro-panel)] text-[var(--noro-cream)]' : 'noro-cta'"
      >
        <img
          v-if="auth.loggedIn.value && identityAvatar(auth.user.value)"
          :src="identityAvatar(auth.user.value)!"
          alt=""
          class="size-5 rounded object-cover"
        >
        <UIcon v-else :name="auth.loggedIn.value ? 'i-lucide-user' : 'i-lucide-log-in'" class="size-3.5" />
        <span>{{ auth.loggedIn.value ? (auth.user.value?.username || t('web-nav-cabinet')) : t('web-nav-sign-in') }}</span>
      </NuxtLink>
    </div>
  </header>
</template>
