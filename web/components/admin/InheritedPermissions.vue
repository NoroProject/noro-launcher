<script setup lang="ts">
defineProps<{
  /** Права, пришедшие по цепочке родителей. */
  permissions: string[]
  /** Имя ближайшего родителя — чтобы было видно, откуда они взялись. */
  parentName: string
}>()
</script>

<template>
  <!-- Только чтение: снять здесь нельзя, право живёт у родителя, и кнопка
       «убрать» либо соврала бы, либо молча правила чужую роль. -->
  <section v-if="permissions.length" class="noro-panel p-5">
    <div class="flex items-center gap-2">
      <UIcon name="i-lucide-git-merge" class="size-4 text-[var(--noro-muted)]" />
      <h2 class="text-xl font-black text-[var(--noro-text)]">Inherited</h2>
    </div>
    <p class="mt-1 text-sm text-[var(--noro-muted)]">
      Comes from <strong class="text-[var(--noro-text)]">{{ parentName || 'the parent role' }}</strong>
      and everything above it. Edit them there.
    </p>

    <div class="mt-4 flex flex-wrap gap-2">
      <code
        v-for="permission in [...permissions].sort()"
        :key="permission"
        class="rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-1.5 font-mono text-xs text-[var(--noro-muted)]"
      >
        {{ permission }}
      </code>
    </div>
  </section>
</template>
