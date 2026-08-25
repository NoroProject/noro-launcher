<script setup lang="ts">
const model = defineModel<string>({ required: true })
const { t } = useT()

const { suggestions } = usePermissionNodes()

const activePreset = ref<'superadmin' | 'fulladmin' | 'seniormod' | 'juniormod' | 'custom'>('fulladmin')
const customRawText = ref(false)

const selectedNodes = computed<string[]>({
  get() {
    return model.value.split('\n').map(s => s.trim()).filter(Boolean)
  },
  set(val: string[]) {
    model.value = val.join('\n')
  }
})

const groups = computed(() => {
  const map = new Map<string, { name: string, title: string }[]>()
  for (const s of suggestions.value) {
    const grp = s.group || 'System'
    const list = map.get(grp) || []
    list.push({ name: s.node, title: s.label || s.node })
    map.set(grp, list)
  }
  return [...map.entries()].map(([title, items]) => ({ title, items }))
})

function applyPreset(type: 'superadmin' | 'fulladmin' | 'seniormod' | 'juniormod' | 'custom') {
  activePreset.value = type
  if (type === 'superadmin') {
    model.value = '*'
  } else if (type === 'fulladmin') {
    model.value = 'noro.admin.*'
  } else if (type === 'seniormod') {
    model.value = [
      'noro.mod.*',
      'noro.admin.users.view',
      'noro.admin.rules.view',
      'noro.admin.audit'
    ].join('\n')
  } else if (type === 'juniormod') {
    model.value = [
      'noro.mod.punish.warn',
      'noro.mod.punish.mute',
      'noro.mod.cases.view',
      'noro.mod.cases.claim',
      'noro.mod.cases.resolve',
      'noro.admin.users.view',
      'noro.admin.rules.view'
    ].join('\n')
  } else if (type === 'custom') {
    model.value = ''
  }
}

function isNodeChecked(node: string): boolean {
  return selectedNodes.value.includes(node) || selectedNodes.value.includes('*') || selectedNodes.value.includes('noro.admin.*')
}

function toggleNode(node: string) {
  activePreset.value = 'custom'
  const list = [...selectedNodes.value]
  const idx = list.indexOf(node)
  if (idx >= 0) {
    list.splice(idx, 1)
  } else {
    list.push(node)
  }
  selectedNodes.value = list
}

function toggleGroup(items: { name: string }[]) {
  activePreset.value = 'custom'
  const current = selectedNodes.value
  const itemNames = items.map(i => i.name)
  const allSelected = itemNames.every(n => current.includes(n))

  if (allSelected) {
    selectedNodes.value = current.filter(n => !itemNames.includes(n))
  } else {
    const next = new Set([...current, ...itemNames])
    selectedNodes.value = [...next]
  }
}
</script>

<template>
  <div class="grid gap-3">
    <div>
      <span class="noro-label mb-1.5 block">{{ t('admin-tokens-preset-title') }}</span>
      <div class="flex flex-wrap gap-2">
        <button
          type="button"
          class="rounded-lg px-2.5 py-1.5 text-xs font-bold transition border"
          :class="activePreset === 'fulladmin' ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)] border-[var(--noro-cream)]' : 'bg-[var(--noro-bg-deep)] text-[var(--noro-muted)] border-[var(--noro-border)] hover:text-[var(--noro-text)]'"
          @click="applyPreset('fulladmin')"
        >
          {{ t('admin-tokens-preset-fulladmin') }}
        </button>
        <button
          type="button"
          class="rounded-lg px-2.5 py-1.5 text-xs font-bold transition border"
          :class="activePreset === 'seniormod' ? 'bg-[var(--noro-blue)] text-[var(--noro-white)] border-[var(--noro-blue)]' : 'bg-[var(--noro-bg-deep)] text-[var(--noro-muted)] border-[var(--noro-border)] hover:text-[var(--noro-text)]'"
          @click="applyPreset('seniormod')"
        >
          {{ t('admin-tokens-preset-senior-mod') }}
        </button>
        <button
          type="button"
          class="rounded-lg px-2.5 py-1.5 text-xs font-bold transition border"
          :class="activePreset === 'juniormod' ? 'bg-[var(--noro-panel)] text-[var(--noro-text)] border-[var(--noro-border)]' : 'bg-[var(--noro-bg-deep)] text-[var(--noro-muted)] border-[var(--noro-border)] hover:text-[var(--noro-text)]'"
          @click="applyPreset('juniormod')"
        >
          {{ t('admin-tokens-preset-junior-mod') }}
        </button>
        <button
          type="button"
          class="rounded-lg px-2.5 py-1.5 text-xs font-bold transition border"
          :class="activePreset === 'superadmin' ? 'bg-[var(--noro-amber)] text-black border-[var(--noro-amber)]' : 'bg-[var(--noro-bg-deep)] text-[var(--noro-muted)] border-[var(--noro-border)] hover:text-[var(--noro-text)]'"
          @click="applyPreset('superadmin')"
        >
          {{ t('admin-tokens-preset-superadmin') }}
        </button>
        <button
          type="button"
          class="rounded-lg px-2.5 py-1.5 text-xs font-bold transition border border-[var(--noro-border)] text-[var(--noro-muted)] hover:text-[var(--noro-text)]"
          @click="applyPreset('custom')"
        >
          {{ t('admin-tokens-preset-custom') }}
        </button>
      </div>
    </div>

    <div class="flex items-center justify-between">
      <span class="noro-label">{{ t('admin-tokens-perms-label') }}</span>
      <button
        type="button"
        class="text-xs text-[var(--noro-blue)] hover:underline"
        @click="customRawText = !customRawText"
      >
        {{ customRawText ? t('admin-tokens-checkbox-toggle') : t('admin-tokens-custom-toggle') }}
      </button>
    </div>

    <div v-if="customRawText">
      <textarea
        v-model="model"
        class="noro-input min-h-32 font-mono text-xs"
        placeholder="noro.admin.*"
        @input="activePreset = 'custom'"
      />
    </div>

    <div v-else class="max-h-72 overflow-y-auto noro-scroll rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-3 grid gap-4">
      <div v-for="grp in groups" :key="grp.title" class="grid gap-2">
        <div class="flex items-center justify-between border-b border-[var(--noro-border)] pb-1">
          <span class="text-xs font-black uppercase text-[var(--noro-cream)]">{{ grp.title }}</span>
          <button
            type="button"
            class="text-[11px] font-semibold text-[var(--noro-muted)] hover:text-[var(--noro-text)]"
            @click="toggleGroup(grp.items)"
          >
            {{ grp.items.every(i => selectedNodes.includes(i.name)) ? t('admin-tokens-deselect-all') : t('admin-tokens-select-all') }}
          </button>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-1.5">
          <label
            v-for="item in grp.items"
            :key="item.name"
            class="flex items-start gap-2 rounded px-2 py-1 transition cursor-pointer hover:bg-[var(--noro-panel)]"
          >
            <input
              type="checkbox"
              class="mt-0.5 size-3.5 rounded border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-cream)]"
              :checked="isNodeChecked(item.name)"
              @change="toggleNode(item.name)"
            >
            <div class="min-w-0 flex-1">
              <div class="text-xs font-semibold text-[var(--noro-text)] leading-snug">{{ item.title }}</div>
              <div class="text-[10px] font-mono text-[var(--noro-muted)] truncate">{{ item.name }}</div>
            </div>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>
