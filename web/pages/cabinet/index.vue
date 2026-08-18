<script setup lang="ts">
import type { UserProfile } from '~/types/api'

const auth = useAuth()
const notify = useNotify()
const { t } = useT()
await auth.loadMe()

const username = ref(auth.user.value?.username || '')
const saving = ref(false)
const saved = ref(false)

const initials = computed(() => (auth.user.value?.username || 'N').slice(0, 1).toUpperCase())
const roles = computed(() => auth.user.value?.roles || [])
const permissions = computed(() => auth.user.value?.permissions || [])
const hasChanges = computed(() => username.value.trim() !== (auth.user.value?.username || ''))

watch(
  () => auth.user.value?.username,
  value => {
    if (value) username.value = value
  }
)

async function saveUsername() {
  const nextName = username.value.trim()
  if (!nextName) return
  saving.value = true
  saved.value = false
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/username', {
      method: 'PUT',
      body: { username: nextName }
    })
    username.value = auth.user.value.username
    saved.value = true
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('cabinet-title')" :subtitle="t('cabinet-subtitle')">
    <div class="grid gap-4">
      <!-- Profile & Username -->
      <section class="noro-panel p-6">
        <div class="flex flex-wrap items-center gap-4">
          <img
            v-if="auth.user.value?.discord_avatar"
            :src="auth.user.value.discord_avatar"
            alt=""
            class="size-16 rounded-[var(--noro-r-sm)] object-cover"
          >
          <div
            v-else
            class="grid size-16 place-items-center rounded-[var(--noro-r-sm)] bg-[var(--noro-magenta)] text-2xl font-bold text-[var(--noro-white)]"
          >
            {{ initials }}
          </div>
          <div class="min-w-0">
            <h2 class="noro-pixel truncate text-2xl text-[var(--noro-cream)]">
              {{ auth.user.value?.username || t('cabinet-player') }}
            </h2>
            <p class="mt-2 truncate text-sm text-[var(--noro-muted)]">
              @{{ auth.user.value?.discord_username || 'discord' }}
            </p>
          </div>
        </div>

        <form class="mt-6 grid gap-3 md:grid-cols-[1fr_auto]" @submit.prevent="saveUsername">
          <label class="min-w-0">
            <span class="noro-label">{{ t('cabinet-mc-name') }}</span>
            <input v-model="username" class="noro-input mt-2" maxlength="16" autocomplete="off">
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">
              {{ t('cabinet-mc-name-hint') }}
            </span>
          </label>
          <AtomButton
            variant="primary"
            icon="i-lucide-save"
            equal
            type="submit"
            :disabled="saving || !hasChanges"
            class="self-start md:mt-[26px]"
          >
            {{ t('cabinet-save') }}
          </AtomButton>
        </form>
        <UAlert
          v-if="saved"
          class="mt-4"
          color="success"
          variant="subtle"
          icon="i-lucide-check"
          :description="t('cabinet-profile-updated')"
        />
      </section>

      <!-- Activity Heatmap -->
      <ActivityHeatmap />

      <!-- Launcher Download -->
      <section class="noro-panel p-6">
        <h2 class="mb-4 font-bold text-[var(--noro-text)]">{{ t('cabinet-launcher-title') }}</h2>
        <LauncherDownload compact />
      </section>

      <!-- Roles & Permissions -->
      <section class="grid gap-4 xl:grid-cols-2">
        <div class="noro-panel p-6">
          <h2 class="noro-label mb-4">{{ t('cabinet-roles-title', { count: roles.length }) }}</h2>
          <div v-if="roles.length" class="grid gap-2">
            <div
              v-for="role in roles"
              :key="role.id"
              class="flex items-center justify-between gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3"
            >
              <div class="flex min-w-0 items-center gap-3">
                <span
                  v-if="role.icon"
                  class="w-4 shrink-0 text-center text-sm"
                  :style="{ color: role.color || 'var(--noro-magenta)' }"
                >{{ role.icon }}</span>
                <span
                  v-else
                  class="size-3 shrink-0 rounded-[2px]"
                  :style="{ backgroundColor: role.color || 'var(--noro-magenta)' }"
                />
                <div class="min-w-0">
                  <div class="truncate text-sm font-bold text-[var(--noro-text)]">
                    {{ role.display_name }}
                  </div>
                  <div class="truncate text-xs text-[var(--noro-muted)]">{{ role.name }}</div>
                </div>
              </div>
              <span class="shrink-0 text-xs text-[var(--noro-muted)]">
                {{ t('cabinet-perms-count', { count: role.permissions.length }) }}
              </span>
            </div>
          </div>
          <EmptyState
            v-else
            icon="i-lucide-shield"
            :title="t('cabinet-roles-none-title')"
            :text="t('cabinet-roles-none-text')"
          />
        </div>

        <div class="noro-panel p-6">
          <h2 class="noro-label mb-4">{{ t('cabinet-direct-perms-title', { count: permissions.length }) }}</h2>
          <div v-if="permissions.length" class="flex flex-wrap gap-2">
            <code
              v-for="perm in permissions"
              :key="perm"
              class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-3 py-2 text-xs text-[var(--noro-blue)]"
            >
              {{ perm }}
            </code>
          </div>
          <EmptyState
            v-else
            icon="i-lucide-key"
            :title="t('cabinet-direct-none-title')"
            :text="t('cabinet-direct-none-text')"
          />
        </div>
      </section>
    </div>
  </NoroShell>
</template>
