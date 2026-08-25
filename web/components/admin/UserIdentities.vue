<script setup lang="ts">
/**
 * Чем игрок входит: привязанные платформы и снятие привязки.
 *
 * Сам игрок отвязывает платформы в кабинете, но к оператору приходят как раз
 * те, кто этого сделать уже не может — потерян доступ к Discord, угнан аккаунт
 * платформы. Поэтому здесь та же операция, но от имени персонала и в журнале.
 *
 * Платформу регистрации не отвязывает и оператор: из неё выведен MC-UUID со
 * всем прогрессом игрока — мастер такой запрос отклонит.
 */

import type { UserIdentity } from '~/types/api'

defineProps<{ identities: UserIdentity[]; canEdit: boolean; busy: string | null }>()
const emit = defineEmits<{ unlink: [provider: string] }>()

const { t } = useT()
</script>

<template>
  <NoroCard :title="t('admin-users-identities')" :subtitle="t('admin-users-identities-hint')" icon="i-lucide-link">
    <div class="grid gap-2">
      <div
        v-for="i in identities"
        :key="i.provider"
        class="flex flex-wrap items-center justify-between gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3"
      >
        <div class="flex min-w-0 items-center gap-3">
          <UIcon :name="providerMeta(i.provider).icon" class="size-5 shrink-0" :style="providerIconStyle(i.provider)" />
          <div class="min-w-0">
            <div class="truncate text-sm font-bold text-[var(--noro-text)]">{{ providerMeta(i.provider).label }}</div>
            <div class="truncate text-xs text-[var(--noro-muted)]">{{ i.username || i.provider_user_id }}</div>
          </div>
        </div>

        <div class="flex shrink-0 items-center gap-3">
          <span class="text-xs text-[var(--noro-muted)]">{{ new Date(i.linked_at).toLocaleDateString() }}</span>
          <span
            v-if="i.is_primary"
            class="text-[10px] font-bold uppercase tracking-wider text-[var(--noro-muted)]"
            :title="t('admin-users-identity-primary-hint')"
          >
            {{ t('cabinet-identity-primary') }}
          </span>
          <AtomButton
            v-else-if="canEdit"
            variant="danger-soft"
            size="sm"
            icon="i-lucide-unlink"
            :loading="busy === `identity-${i.provider}`"
            @click="emit('unlink', i.provider)"
          >
            {{ t('cabinet-identity-unlink') }}
          </AtomButton>
        </div>
      </div>

      <EmptyState
        v-if="!identities.length"
        icon="i-lucide-unlink"
        :title="t('admin-users-identities-none')"
        :text="t('admin-users-identities-none-hint')"
      />
    </div>
  </NoroCard>
</template>
