<script setup lang="ts">
const auth = useAuth()
await auth.loadMe()

const sessionRows = computed(() => [
  ['User', auth.user.value?.username || 'Player'],
  ['Discord', auth.user.value?.discord_username || 'Unknown'],
  ['Minecraft UUID', auth.user.value?.uuid || 'Not loaded']
])
</script>

<template>
  <!--
    Блок «Master Server» убран: адрес мастера — внутренняя настройка развёртывания,
    игроку он ничего не даёт, а упоминание NUXT_PUBLIC_MASTER_URL выносило наружу
    кухню сборки. В лаунчере эту же настройку убрали по той же причине.
  -->
  <NoroShell title="SETTINGS" subtitle="Your account">
    <template #actions>
      <NuxtLink to="/cabinet" class="noro-btn noro-btn-secondary">
        <UIcon name="i-lucide-arrow-left" class="size-4" />Cabinet
      </NuxtLink>
    </template>

    <div class="mx-auto grid w-full max-w-2xl gap-4">
      <section class="noro-panel p-6">
        <h2 class="noro-label mb-4">Account</h2>
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
        <h2 class="noro-label mb-2">Session</h2>
        <p class="mb-4 text-sm text-[var(--noro-muted)]">
          Signing out only affects this browser. The launcher keeps its own session.
        </p>
        <button type="button" class="noro-btn noro-btn-secondary" @click="auth.signOut()">
          <UIcon name="i-lucide-log-out" class="size-4" />Sign out
        </button>
      </section>
    </div>
  </NoroShell>
</template>
