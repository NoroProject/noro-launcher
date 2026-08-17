<script setup lang="ts">
const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const sessionRows = computed(() => [
  [t('cabinet-player'), auth.user.value?.username || 'Player'],
  ['Discord', auth.user.value?.discord_username || 'Unknown'],
  ['Minecraft UUID', auth.user.value?.uuid || 'Not loaded']
])
</script>

<template>
  <NoroShell :title="t('nav-cabinet-settings')" subtitle="Your account">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" to="/cabinet">{{ t('nav-cabinet-home') }}</AtomButton>
    </template>

    <div class="grid gap-4">
      <section class="noro-panel p-6">
        <h2 class="noro-label mb-4">{{ t('cabinet-settings-account') }}</h2>
        <div class="grid gap-2 text-sm">
          <div
            v-for="[label, value] in sessionRows"
            :key="label"
            class="grid gap-1 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3 md:grid-cols-[160px_1fr] md:items-center"
          >
            <span class="noro-label">{{ label }}</span>
            <span class="break-all text-[var(--noro-text)]">{{ value }}</span>
          </div>
        </div>
      </section>

      <section class="noro-panel p-6">
        <h2 class="noro-label mb-2">{{ t('cabinet-settings-session') }}</h2>
        <p class="mb-4 text-sm text-[var(--noro-muted)]">
          {{ t('cabinet-settings-signout-hint') }}
        </p>
        <AtomButton variant="secondary" icon="i-lucide-log-out" @click="auth.signOut()">{{ t('profile-sign-out') }}</AtomButton>
      </section>
    </div>
  </NoroShell>
</template>
