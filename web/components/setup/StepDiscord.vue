<script setup lang="ts">
const props = defineProps<{ apiUrl: string; secretPresent: boolean }>()
const model = defineModel<Record<string, string>>({ required: true })
const emit = defineEmits<{ next: [] }>()

/** Ровно то, что нужно вписать в портал Discord. */
const redirects = computed(() => [
  `${props.apiUrl.replace(/\/$/, '')}/auth/discord/callback`,
  `${props.apiUrl.replace(/\/$/, '')}/auth/discord/launcher/callback`,
])
</script>

<template>
  <div class="grid gap-4">
    <label class="block">
      <span class="noro-label mb-1.5 block">Client ID</span>
      <input v-model="model.discord_client_id" class="noro-input w-full font-mono" placeholder="1512048650258219089">
    </label>

    <UAlert
      :color="props.secretPresent ? 'success' : 'warning'"
      variant="subtle"
      :icon="props.secretPresent ? 'i-lucide-check' : 'i-lucide-triangle-alert'"
      title="DISCORD_CLIENT_SECRET"
      :description="props.secretPresent
        ? 'Present in the environment — nothing to do here.'
        : 'Not set. The master cannot write it for you: secrets live in the environment, never in the database. Add it to .env and restart.'"
    />

    <div>
      <span class="noro-label mb-1.5 block">Redirect URIs for the Discord portal</span>
      <pre class="noro-scroll overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">{{ redirects.join('\n') }}</pre>
    </div>

    <div>
      <AtomButton icon="i-lucide-arrow-right" @click="emit('next')">Continue</AtomButton>
    </div>
  </div>
</template>
