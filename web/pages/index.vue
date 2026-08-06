<script setup lang="ts">
const auth = useAuth()
const target = computed(() => (auth.loggedIn.value ? '/cabinet' : '/login'))

const features = [
  ['Players', 'One login, clean server access, verified files, and a cabinet that keeps the launcher personal.'],
  ['Servers', 'Builds, roles, news, skins, and release control live in one quiet admin surface.'],
  ['Safety', 'Signed manifests and file checks keep launches predictable before the game opens.']
]

const steps = [
  ['Sign in', 'Discord account connects the cabinet and launcher identity.'],
  ['Choose a server', 'Players see only the worlds they can access.'],
  ['Sync files', 'Required and optional mods are checked before launch.'],
  ['Play', 'The launcher opens Minecraft with the right build ready.']
]
</script>

<template>
  <div class="min-h-screen overflow-hidden">
    <header class="mx-auto flex max-w-7xl items-center justify-between px-5 py-6">
      <NuxtLink to="/" class="flex items-center gap-3">
        <div class="grid size-12 place-items-center rounded-lg">
            <img src="/icon.png"/>
        </div>
        <div>
          <div class="noro-pixel text-xl uppercase text-[var(--noro-cream)]">NORO</div>
          <div class="text-xs font-black uppercase tracking-wider text-[var(--noro-muted)]">launcher</div>
        </div>
      </NuxtLink>
      <NuxtLink :to="target" class="noro-cta px-5 py-3 text-sm uppercase">
        {{ auth.loggedIn.value ? 'Open cabinet' : 'Sign in' }}
      </NuxtLink>
    </header>

    <main>
      <section class="mx-auto grid max-w-7xl gap-12 px-5 pb-16 pt-10 lg:grid-cols-[1fr_520px] lg:items-center">
        <div>
          <div class="mb-5 inline-flex rounded-lg bg-[var(--noro-panel)] px-4 py-2 text-xs font-black uppercase tracking-wider text-[var(--noro-blue)]">
            Private launcher for modded servers
          </div>
          <h1 class="noro-pixel max-w-4xl text-5xl leading-none text-[var(--noro-cream)] md:text-7xl">
            NORO LAUNCHER
          </h1>
          <p class="mt-6 max-w-2xl text-lg font-medium leading-8 text-[var(--noro-text)]">
            A controlled launch experience for Minecraft communities: Discord login, file sync, permissions, news, skins, and signed builds in one place.
          </p>
          <div class="mt-8 flex flex-wrap gap-3">
            <NuxtLink :to="target" class="noro-cta px-6 py-3">
              {{ auth.loggedIn.value ? 'OPEN CABINET' : 'START WITH DISCORD' }}
            </NuxtLink>
            <AtomButton variant="secondary" icon="i-lucide-shirt" to="/skin">Skin cabinet</AtomButton>
          </div>
        </div>

        <!-- Место под рендер лаунчера. Раньше здесь был выдуманный «Noro SMP»
             с несуществующей сборкой и цветными плашками: макет выглядел как
             живые данные и обещал то, чего на странице нет. -->
        <div
          class="noro-panel relative grid min-h-[420px] place-items-center overflow-hidden bg-[var(--noro-bg-deep)]"
        >
          <div class="flex flex-col items-center gap-4 px-6 text-center">
            <div
              class="grid size-16 place-items-center rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel)]"
            >
              <UIcon name="i-lucide-image" class="size-7 text-[var(--noro-muted)]" />
            </div>
            <div class="noro-label">Launcher preview</div>
          </div>
        </div>
      </section>

      <section class="mx-auto max-w-7xl px-5 pb-4">
        <LauncherDownload />
      </section>

      <section class="bg-[var(--noro-panel)] px-5 py-16">
        <div class="mx-auto grid max-w-7xl gap-4 md:grid-cols-3">
          <article v-for="[title, text] in features" :key="title" class="rounded-lg bg-[var(--noro-input)] p-6 transition-all duration-200 hover:scale-[1.02]">
            <h2 class="text-2xl font-black text-[var(--noro-cream)]">{{ title }}</h2>
            <p class="mt-4 text-base font-medium leading-7 text-[var(--noro-text)]">{{ text }}</p>
          </article>
        </div>
      </section>

      <section class="mx-auto max-w-7xl px-5 py-16">
        <div class="mb-8 max-w-2xl">
          <div class="text-sm font-black uppercase tracking-wider text-[var(--noro-blue)]">Launch flow</div>
          <h2 class="mt-3 text-4xl font-black text-[var(--noro-cream)]">From Discord to Minecraft in four clean steps</h2>
        </div>
        <div class="grid gap-3 md:grid-cols-4">
          <article v-for="([title, text], index) in steps" :key="title" class="rounded-lg bg-[var(--noro-panel)] p-5">
            <div class="text-3xl font-black text-[var(--noro-blue)]">0{{ index + 1 }}</div>
            <h3 class="mt-5 text-xl font-black text-[var(--noro-text)]">{{ title }}</h3>
            <p class="mt-3 text-sm font-medium leading-6 text-[var(--noro-muted)]">{{ text }}</p>
          </article>
        </div>
      </section>

      <section class="px-5 pb-16">
        <div class="mx-auto flex max-w-7xl flex-col gap-5 rounded-lg bg-[var(--noro-cream)] p-6 text-[var(--noro-on-cream)] md:flex-row md:items-center md:justify-between md:p-8">
          <div>
            <h2 class="text-3xl font-black">Ready for the next launch?</h2>
            <p class="mt-2 max-w-2xl text-base font-bold">Open the cabinet, connect Discord, and keep the project moving from one controlled surface.</p>
          </div>
          <AtomButton
            variant="dark"
            :to="target"
          >
            {{ auth.loggedIn.value ? 'Open cabinet' : 'Sign in' }}
          </AtomButton>
        </div>
      </section>
    </main>
  </div>
</template>
