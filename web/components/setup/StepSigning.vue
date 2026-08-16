<script setup lang="ts">
import type { SigningKey } from '~/types/setup'

const props = defineProps<{ keyPresent: boolean }>()
const emit = defineEmits<{ generate: []; next: [] }>()
const generated = defineModel<SigningKey | null>({ required: true })

const copied = ref(false)

async function copy() {
  if (!generated.value) return
  await navigator.clipboard.writeText(generated.value.env_line)
  copied.value = true
}
</script>

<template>
  <div class="grid gap-4">
    <p class="text-sm text-[var(--noro-muted)]">
      Manifests are signed with an ed25519 key so a patched launcher cannot hand
      a player a build nobody published. The private half never touches the
      database or the log — it is shown here once and nowhere else.
    </p>

    <UAlert
      v-if="props.keyPresent"
      color="success"
      variant="subtle"
      icon="i-lucide-check"
      title="NORO_SIGNING_KEY"
      description="Already set in the environment. Generating a new one would invalidate every build signed so far."
    />

    <template v-else>
      <div>
        <AtomButton icon="i-lucide-sparkles" @click="emit('generate')">Generate a key</AtomButton>
      </div>

      <div v-if="generated" class="grid gap-3">
        <UAlert
          color="warning"
          variant="subtle"
          icon="i-lucide-triangle-alert"
          title="Copy this now"
          description="It is not stored anywhere. Lose it and you generate a new one — which invalidates every signature made with the old key."
        />
        <div>
          <span class="noro-label mb-1.5 block">Add to .env</span>
          <pre class="noro-scroll overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">{{ generated.env_line }}</pre>
        </div>
        <div>
          <span class="noro-label mb-1.5 block">Public key — build the launcher with it</span>
          <pre class="noro-scroll overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">NORO_MANIFEST_PUBKEY={{ generated.public_hex }}</pre>
        </div>
        <div>
          <AtomButton :icon="copied ? 'i-lucide-check' : 'i-lucide-copy'" variant="dark" @click="copy">
            {{ copied ? 'Copied' : 'Copy the .env line' }}
          </AtomButton>
        </div>
      </div>
    </template>

    <div>
      <AtomButton icon="i-lucide-arrow-right" @click="emit('next')">Continue</AtomButton>
    </div>
  </div>
</template>
