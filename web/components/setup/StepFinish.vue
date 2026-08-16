<script setup lang="ts">
const props = defineProps<{ envBlock: string; secureContext: boolean; done: boolean }>()
const emit = defineEmits<{ finish: [] }>()

const copied = ref(false)

async function copy() {
  await navigator.clipboard.writeText(props.envBlock)
  copied.value = true
}
</script>

<template>
  <div v-if="props.done" class="grid gap-4">
    <UAlert
      color="success"
      variant="subtle"
      icon="i-lucide-check"
      title="Setup complete"
      description="Restart the master so it picks up the new settings and any secrets you added. The setup token is burned — this page will not open again."
    />
    <pre class="noro-scroll overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">docker compose restart master</pre>
  </div>

  <div v-else class="grid gap-4">
    <UAlert
      v-if="!props.secureContext"
      color="warning"
      variant="subtle"
      icon="i-lucide-shield-off"
      title="Passkeys will not work on this address"
      description="WebAuthn refuses http:// on anything but localhost. Until this instance has a domain and TLS, recovery codes are the only way back in — keep them."
    />

    <div>
      <span class="noro-label mb-1.5 block">Your .env</span>
      <pre class="noro-scroll max-h-80 overflow-auto rounded bg-[var(--noro-input)] p-3 text-xs">{{ props.envBlock }}</pre>
    </div>

    <div class="flex flex-wrap gap-2">
      <AtomButton :icon="copied ? 'i-lucide-check' : 'i-lucide-copy'" variant="dark" @click="copy">
        {{ copied ? 'Copied' : 'Copy' }}
      </AtomButton>
      <AtomButton icon="i-lucide-flag" @click="emit('finish')">Finish setup</AtomButton>
    </div>
  </div>
</template>
