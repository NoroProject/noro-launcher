<script setup lang="ts">
const props = defineProps<{ tokenPath?: string }>()
const emit = defineEmits<{ done: [token: string] }>()

const value = ref('')
</script>

<template>
  <div class="grid gap-4">
    <p class="text-sm text-[var(--noro-muted)]">
      The master printed a one-time token at startup and wrote it to
      <code class="rounded bg-[var(--noro-input)] px-1.5 py-0.5">{{ props.tokenPath || '$NORO_DATA_DIR/setup-token.txt' }}</code>.
      Whoever can read that file is whoever sets this instance up — on an empty
      instance there is nothing else to authenticate against.
    </p>

    <label class="block">
      <span class="noro-label mb-1.5 block">Setup token</span>
      <input
        v-model="value"
        class="noro-input w-full font-mono"
        placeholder="e68f8484f1fb3395…"
        autofocus
        @keyup.enter="emit('done', value)"
      >
    </label>

    <div>
      <AtomButton icon="i-lucide-key-round" :disabled="!value.trim()" @click="emit('done', value)">
        Continue
      </AtomButton>
    </div>
  </div>
</template>
