<script setup lang="ts">
const props = defineProps<{ secureContext: boolean }>()
const emit = defineEmits<{ create: [username: string]; next: [] }>()
const codes = defineModel<string[] | null>({ required: true })

const username = ref('')
const saved = ref(false)

function download() {
  if (!codes.value) return
  const body = [
    'Recovery codes for this Noro instance.',
    'Each one works once. Keep them somewhere you can reach without this site.',
    '',
    ...codes.value,
  ].join('\n')
  const url = URL.createObjectURL(new Blob([body], { type: 'text/plain' }))
  const a = document.createElement('a')
  a.href = url
  a.download = 'noro-recovery-codes.txt'
  a.click()
  URL.revokeObjectURL(url)
  saved.value = true
}
</script>

<template>
  <div class="grid gap-4">
    <p class="text-sm text-[var(--noro-muted)]">
      The operator account. It has no Discord behind it and, by default, no
      right to join the game — it exists to run this instance, not to play on it.
    </p>

    <template v-if="!codes">
      <label class="block">
        <span class="noro-label mb-1.5 block">Username</span>
        <input
          v-model="username"
          class="noro-input w-full"
          placeholder="operator"
          maxlength="16"
        >
        <span class="mt-1 block text-xs text-[var(--noro-muted)]">
          Up to 16 characters: letters, digits and underscore.
        </span>
      </label>
      <div>
        <AtomButton icon="i-lucide-user-plus" :disabled="!username.trim()" @click="emit('create', username)">
          Create the account
        </AtomButton>
      </div>
    </template>

    <template v-else>
      <UAlert
        :color="props.secureContext ? 'warning' : 'error'"
        variant="subtle"
        icon="i-lucide-key-round"
        title="Save these codes now"
        :description="props.secureContext
          ? 'Each code works once. They are shown here and never again.'
          : 'Each code works once, and on a plain http:// address they are your only way in — passkeys refuse to bind here. They are shown once and never again.'"
      />

      <ol class="grid gap-1 rounded bg-[var(--noro-input)] p-4 font-mono text-sm sm:grid-cols-2">
        <li v-for="code in codes" :key="code">{{ code }}</li>
      </ol>

      <div class="flex flex-wrap gap-2">
        <AtomButton :icon="saved ? 'i-lucide-check' : 'i-lucide-download'" variant="dark" @click="download">
          {{ saved ? 'Downloaded' : 'Download as a file' }}
        </AtomButton>
        <AtomButton icon="i-lucide-arrow-right" @click="emit('next')">Continue</AtomButton>
      </div>
    </template>
  </div>
</template>
