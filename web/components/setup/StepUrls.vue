<script setup lang="ts">
const model = defineModel<Record<string, string>>({ required: true })
const emit = defineEmits<{ next: [] }>()

/** Без этих двух инстанс нельзя завершить: манифесты уехали бы в никуда. */
const ready = computed(
  () => !!model.value.public_url?.trim() && !!model.value.web_url?.trim()
)
</script>

<template>
  <div class="grid gap-4">
    <label class="block">
      <span class="noro-label mb-1.5 block">Instance name</span>
      <input v-model="model.instance_name" class="noro-input w-full" placeholder="Noro Network">
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">API URL</span>
      <input v-model="model.public_url" class="noro-input w-full" placeholder="https://api.example.dev">
      <span class="mt-1 block text-xs text-[var(--noro-muted)]">
        Where the master answers. It ends up in every manifest a player downloads.
      </span>
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">Site URL</span>
      <input v-model="model.web_url" class="noro-input w-full" placeholder="https://example.dev">
      <span class="mt-1 block text-xs text-[var(--noro-muted)]">
        Where this page lives. Passkeys bind to this domain permanently.
      </span>
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">Allowed CORS origins</span>
      <input v-model="model.allowed_origins" class="noro-input w-full" placeholder="https://example.dev, https://admin.example.dev">
      <span class="mt-1 block text-xs text-[var(--noro-muted)]">
        Comma-separated. Left empty, the master accepts any origin — fine locally, not in production.
      </span>
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">CDN URL for files <span class="text-[var(--noro-muted)]">— optional</span></span>
      <input v-model="model.files_cdn_url" class="noro-input w-full" placeholder="https://cdn.example.dev/files">
    </label>

    <div>
      <AtomButton icon="i-lucide-arrow-right" :disabled="!ready" @click="emit('next')">
        Save and continue
      </AtomButton>
    </div>
  </div>
</template>
