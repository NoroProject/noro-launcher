<script setup lang="ts">
import type { BuildFileRow } from "~/types/api";

const pathsForm = defineModel<{ unmanaged: string; userManaged: string }>("pathsForm", { required: true });
const props = withDefaults(defineProps<{ files?: BuildFileRow[] | null; busy?: string | null }>(), {});
defineEmits<{ save: [] }>();

const showModal = ref(false);

const ignoredCount = computed(() => pathsForm.value.unmanaged.split("\n").filter(l => l.trim()).length);
const userCount = computed(() => pathsForm.value.userManaged.split("\n").filter(l => l.trim()).length);

function openBrowser() { showModal.value = true; }
function applyRules(unm: string, usr: string) {
  pathsForm.value.unmanaged = unm;
  pathsForm.value.userManaged = usr;
}
</script>

<template>
  <section class="noro-panel overflow-hidden">
    <div class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4">
      <div class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-blue)]">
        <UIcon name="i-lucide-route" class="size-5" />
      </div>
      <div class="flex-1">
        <div class="font-bold text-[var(--noro-text)]">Path Rules</div>
        <div class="text-[10px] uppercase tracking-wider text-[var(--noro-muted)]">Sync exclusions &amp; overrides</div>
      </div>
      <div class="text-right text-[10px] font-mono text-[var(--noro-muted)]">
        {{ ignoredCount + userCount }} rules
      </div>
    </div>

    <div class="p-5 grid gap-4">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-lg border border-[var(--noro-border)] bg-black/20 p-3">
          <div class="flex items-center gap-2 text-[var(--noro-blue)]">
            <UIcon name="i-lucide-eye-off" class="size-4" />
            <span class="text-xs font-bold uppercase tracking-widest">Ignored</span>
          </div>
          <div class="mt-1 text-2xl font-black text-[var(--noro-text)]">{{ ignoredCount }}</div>
          <div class="text-[10px] text-[var(--noro-muted)]">Never touched by sync</div>
        </div>
        <div class="rounded-lg border border-[var(--noro-border)] bg-black/20 p-3">
          <div class="flex items-center gap-2 text-[var(--noro-magenta)]">
            <UIcon name="i-lucide-user" class="size-4" />
            <span class="text-xs font-bold uppercase tracking-widest">User Overrides</span>
          </div>
          <div class="mt-1 text-2xl font-black text-[var(--noro-text)]">{{ userCount }}</div>
          <div class="text-[10px] text-[var(--noro-muted)]">Seeded once, player edits kept</div>
        </div>
      </div>

      <AtomButton
        icon="i-lucide-folder-open"
        variant="primary"
        block
        @click="openBrowser"
      >
        Browse &amp; Edit Rules
      </AtomButton>

      <AtomButton
        icon="i-lucide-save"
        variant="primary"
        :loading="busy === 'paths'"
        block
        @click="$emit('save')"
      >
        Save Path Rules
      </AtomButton>
    </div>
  </section>

  <PathRulesModal
    v-model="showModal"
    :unmanaged="pathsForm.unmanaged"
    :user-managed="pathsForm.userManaged"
    :files="props.files"
    @apply="applyRules"
  />
</template>
