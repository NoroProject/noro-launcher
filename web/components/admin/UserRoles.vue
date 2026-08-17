<script setup lang="ts">
import type { Role } from '~/types/api'

const props = defineProps<{
  roles: Role[];
  userRoles: Role[];
  busy: string | null;
}>()
const emit = defineEmits<{
  add: [roleId: string];
  remove: [roleId: string];
}>()

const { t } = useT()
const selectedRole = ref('')
const freeRoles = computed(() => props.roles.filter(role =>
  !props.userRoles.some(item => item.id === role.id)
))

function addRole() {
  if (!selectedRole.value) return
  emit('add', selectedRole.value)
  selectedRole.value = ''
}
</script>

<template>
  <div class="noro-panel p-5">
    <h2 class="mb-4 text-xl font-black text-[var(--noro-text)]">{{ t('admin-roles-title') }}</h2>
    <div class="mb-4 flex gap-2">
      <NoroSelect v-model="selectedRole">
        <option value="">{{ t('admin-roles-select') }}</option>
        <option v-for="role in freeRoles" :key="role.id" :value="role.id">{{ role.display_name }}</option>
      </NoroSelect>
      <AtomButton variant="primary" :loading="busy === 'role'" icon="i-lucide-plus" @click="addRole" />
    </div>
    <div class="grid gap-2">
      <div v-for="role in userRoles" :key="role.id" class="flex items-center justify-between rounded-lg bg-[var(--noro-input)] px-3 py-2">
        <span class="font-bold text-[var(--noro-text)]">{{ role.display_name }}</span>
        <AtomButton variant="danger" :loading="busy === `role-${role.id}`" icon="i-lucide-x" size="sm" @click="emit('remove', role.id)" />
      </div>
    </div>
  </div>
</template>
