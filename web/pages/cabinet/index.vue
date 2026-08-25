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

/** Что мастер не принял, по полям. Пусто — ошибок формы нет. */
const fieldErrors = ref<Record<string, string>>({})

async function saveUsername() {
  const nextName = username.value.trim()
  if (!nextName) return
  saving.value = true
  saved.value = false
  fieldErrors.value = {}
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/username', {
      method: 'PUT',
      body: { username: nextName }
    })
    username.value = auth.user.value.username
    saved.value = true
    notify.ok()
  } catch (e) {
    // Правило про ник приезжает привязанным к полю, а не абзацем под формой:
    // подпись у самого поля показывает, что именно исправлять.
    fieldErrors.value = apiFieldMessages(e)
    if (!Object.keys(fieldErrors.value).length) notify.fail(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('cabinet-title')" :subtitle="t('cabinet-subtitle')">
    <div class="grid gap-4">
      <NoroCard :title="t('cabinet-profile-title')" icon="i-lucide-user-round">
        <div class="flex flex-wrap items-center gap-4">
          <img
            v-if="identityAvatar(auth.user.value)"
            :src="identityAvatar(auth.user.value)!"
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
              {{ identityHandle(auth.user.value) ? `@${identityHandle(auth.user.value)}` : auth.user.value?.username }}
            </p>
          </div>
        </div>

        <form class="mt-5 grid gap-3 md:grid-cols-[1fr_auto]" @submit.prevent="saveUsername">
          <label class="min-w-0">
            <span class="noro-label">{{ t('cabinet-mc-name') }}</span>
            <input
              v-model="username"
              class="noro-input mt-2"
              :class="fieldErrors.username && 'border-[var(--noro-magenta)]'"
              maxlength="16"
              autocomplete="off"
            >
            <span
              class="mt-2 block text-xs"
              :class="fieldErrors.username ? 'text-[var(--noro-magenta)]' : 'text-[var(--noro-muted)]'"
            >
              {{ fieldErrors.username || t('cabinet-mc-name-hint') }}
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
      </NoroCard>

      <!-- Activity Heatmap -->
      <ActivityHeatmap />

      <NoroCard :title="t('cabinet-launcher-title')" icon="i-lucide-download">
        <LauncherDownload compact />
      </NoroCard>

      <!-- Roles & Permissions -->
      <section class="grid gap-4 xl:grid-cols-2">
        <NoroCard :title="t('cabinet-roles-card')" icon="i-lucide-shield">
          <template #actions>
            <span class="text-xs text-[var(--noro-muted)]">{{ roles.length }}</span>
          </template>
          <div v-if="roles.length" class="grid gap-2">
            <div
              v-for="role in roles"
              :key="role.id"
              class="flex items-center justify-between gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3"
            >
              <div class="flex min-w-0 items-center gap-3">
                <!-- Плашка вместо иконки и кружка: игрок уже видел её в чате,
                     и узнаёт роль по ней быстрее, чем по цвету квадратика. -->
                <RoleBadge :role-id="role.id" :color="role.color" :height="14" />
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
        </NoroCard>

        <NoroCard :title="t('cabinet-direct-card')" icon="i-lucide-key">
          <template #actions>
            <span class="text-xs text-[var(--noro-muted)]">{{ permissions.length }}</span>
          </template>
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
        </NoroCard>
      </section>
    </div>
  </NoroShell>
</template>
