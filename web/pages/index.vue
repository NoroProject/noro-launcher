<script setup lang="ts">
definePageMeta({ layout: 'public' })

const auth = useAuth()
const { t } = useT()
const brand = usePublicSettings()
const { servers, online, pending } = usePublicServers()

const target = computed(() => (auth.loggedIn.value ? '/cabinet' : '/login'))
/** На главной — витрина, а не полный список: остальное на своей странице. */
const featured = computed(() => servers.value.slice(0, 3))

const steps = computed(() => ['signin', 'server', 'sync', 'play'].map(step => ({
  key: step,
  title: t(`web-home-step-${step}-title`),
  text: t(`web-home-step-${step}-text`),
})))

const features = computed(() => [
  ['builds', 'i-lucide-package-check'],
  ['identity', 'i-lucide-users'],
  ['rules', 'i-lucide-scale'],
].map(([key, icon]) => ({
  key,
  icon,
  title: t(`web-home-feature-${key}-title`),
  text: t(`web-home-feature-${key}-text`),
})))
</script>

<template>
  <div>
    <!-- Hero -->
    <section class="mx-auto grid max-w-7xl gap-10 px-5 pb-12 pt-12 lg:grid-cols-[1fr_460px] lg:items-center">
      <!-- Left Column: Copy, CTAs & Download -->
      <div>
        <div class="mb-5 inline-flex items-center gap-2 rounded-[var(--noro-r-sm)] bg-[var(--noro-panel)] px-4 py-2 text-xs font-black uppercase tracking-wider text-[var(--noro-blue)]">
          <span class="size-2 rounded-full" :class="online ? 'bg-[var(--noro-green)] shadow-[0_0_8px_var(--noro-green)]' : 'bg-[var(--noro-muted)]'" />
          {{ t('web-home-online-now', { count: online }) }}
        </div>
        <!-- Название инстанса, а не «Noro Launcher» намертво: заголовок главной
             — часть брендинга, и правится он там же, где логотип. -->
        <h1 class="noro-pixel text-4xl leading-none text-[var(--noro-cream)] md:text-6xl">
          {{ brand.instance_name }}
        </h1>
        <p class="mt-6 max-w-2xl text-lg font-medium leading-8 text-[var(--noro-text)]">
          {{ t('web-home-lead') }}
        </p>
        <div class="mt-8 flex flex-wrap gap-3">
          <NuxtLink :to="target" class="noro-cta px-6 py-3">
            {{ auth.loggedIn.value ? t('web-home-cta-cabinet') : t('web-home-cta-sign-in') }}
          </NuxtLink>
          <AtomButton variant="secondary" icon="i-lucide-server" :to="link.servers()">
            {{ t('web-nav-servers') }}
          </AtomButton>
          <AtomButton variant="ghost" icon="i-lucide-book-open" :to="link.rules()">
            {{ t('web-nav-rules') }}
          </AtomButton>
        </div>

        <!-- Download Widget -->
        <div class="mt-8 max-w-xl rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel)] p-4 shadow-lg">
          <LauncherDownload compact />
        </div>
      </div>

      <!-- Right Column: Pure 3D Minecraft Character Render Showcase -->
      <PublicHeroArt
        :src="brand.hero_image_url || '/hero-character.jpg'"
        :transparent="brand.hero_image_transparent"
      />
    </section>

    <!-- Серверы -->
    <section class="mx-auto max-w-7xl px-5 py-12">
      <div class="mb-6 flex flex-wrap items-end justify-between gap-4">
        <div>
          <div class="text-sm font-black uppercase tracking-wider text-[var(--noro-blue)]">
            {{ t('web-home-worlds-eyebrow') }}
          </div>
          <h2 class="mt-2 text-3xl font-black text-[var(--noro-cream)]">
            {{ t('web-home-worlds-title') }}
          </h2>
        </div>
        <AtomButton variant="secondary" icon-right="i-lucide-arrow-right" :to="link.servers()">
          {{ t('web-home-worlds-all') }}
        </AtomButton>
      </div>

      <div v-if="pending && !featured.length" class="noro-panel grid place-items-center p-12 text-[var(--noro-muted)]">
        <UIcon name="i-lucide-loader-2" class="size-6 animate-spin" />
      </div>
      <EmptyState
        v-else-if="!featured.length"
        icon="i-lucide-server-off"
        :title="t('web-servers-empty-title')"
        :text="t('web-servers-empty-short')"
      />
      <div v-else class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        <PublicServerCard v-for="server in featured" :key="server.id" :server="server" />
      </div>
    </section>

    <!-- Что даёт проект -->
    <section class="bg-[var(--noro-panel)] px-5 py-16">
      <div class="mx-auto grid max-w-7xl gap-4 md:grid-cols-3">
        <article v-for="item in features" :key="item.key" class="rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-6">
          <UIcon :name="item.icon" class="size-7 text-[var(--noro-cream)]" />
          <h3 class="mt-4 text-xl font-black text-[var(--noro-cream)]">{{ item.title }}</h3>
          <p class="mt-3 text-sm font-medium leading-6 text-[var(--noro-text)]">{{ item.text }}</p>
        </article>
      </div>
    </section>

    <!-- Как начать -->
    <section class="mx-auto max-w-7xl px-5 py-16">
      <div class="mb-8 max-w-2xl">
        <div class="text-sm font-black uppercase tracking-wider text-[var(--noro-blue)]">
          {{ t('web-home-flow-eyebrow') }}
        </div>
        <h2 class="mt-3 text-3xl font-black text-[var(--noro-cream)]">
          {{ t('web-home-flow-title') }}
        </h2>
      </div>
      <div class="grid gap-3 md:grid-cols-4">
        <article v-for="(step, index) in steps" :key="step.key" class="noro-panel p-5">
          <div class="noro-pixel text-2xl text-[var(--noro-blue)]">0{{ index + 1 }}</div>
          <h3 class="mt-4 text-lg font-black text-[var(--noro-text)]">{{ step.title }}</h3>
          <p class="mt-2 text-sm font-medium leading-6 text-[var(--noro-muted)]">{{ step.text }}</p>
        </article>
      </div>
    </section>

    <!-- Правила -->
    <section class="px-5 pb-16">
      <div class="mx-auto flex max-w-7xl flex-col gap-5 rounded-[var(--noro-r-sm)] bg-[var(--noro-cream)] p-6 text-[var(--noro-on-cream)] md:flex-row md:items-center md:justify-between md:p-8">
        <div>
          <h2 class="text-2xl font-black">{{ t('web-home-rules-title') }}</h2>
          <p class="mt-2 max-w-2xl text-base font-bold">
            {{ t('web-home-rules-text') }}
          </p>
        </div>
        <AtomButton variant="dark" icon="i-lucide-book-open" :to="link.rules()">
          {{ t('web-home-rules-cta') }}
        </AtomButton>
      </div>
    </section>
  </div>
</template>
