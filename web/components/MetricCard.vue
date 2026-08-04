<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  detail?: string
  icon?: string
  tone?: 'blue' | 'green' | 'amber' | 'red' | 'magenta'
}>()
</script>

<template>
  <div class="noro-panel group p-5 transition-all duration-200 hover:scale-[1.02]">
    <div class="flex items-start justify-between gap-3">
      <div>
        <div class="text-xs font-black uppercase tracking-wider text-[var(--noro-muted)]">{{ label }}</div>
        <div
          class="mt-2 text-3xl font-black"
          :class="{
            'text-[var(--noro-blue)]': !tone || tone === 'blue',
            'text-[var(--noro-magenta)]': tone === 'magenta',
            'text-[var(--noro-green)]': tone === 'green',
            'text-[var(--noro-amber)]': tone === 'amber',
            'text-[var(--noro-danger)]': tone === 'red'
          }"
        >
          {{ value }}
        </div>
      </div>
      <div
        v-if="icon"
        class="grid size-12 place-items-center rounded-lg transition-transform duration-200 group-hover:scale-110"
        :class="{
          'bg-[var(--noro-input)] text-[var(--noro-blue)]': !tone || tone === 'blue',
          'bg-[var(--noro-input)] text-[var(--noro-magenta)]': tone === 'magenta',
          'bg-[var(--noro-input)] text-[var(--noro-green)]': tone === 'green',
          'bg-[var(--noro-input)] text-[var(--noro-amber)]': tone === 'amber',
          'bg-[var(--noro-input)] text-[var(--noro-danger)]': tone === 'red'
        }"
      >
        <UIcon :name="icon" class="size-5" />
      </div>
    </div>
    <p v-if="detail" class="mt-3 text-sm text-[var(--noro-muted)]">{{ detail }}</p>
  </div>
</template>
