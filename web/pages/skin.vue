<script setup lang="ts">
import type { UserProfile } from '~/types/api'
import type { CapeRow } from '~/types/cape'

const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const MAX_BYTES = 256 * 1024

/** Ответ `/api/me/skin-presets` — поля как в `SkinPresetItem` на мастере. */
interface SavedSkin {
  id: string
  name: string
  skin_url: string
  skin_slim: boolean
}

const file = ref<File | null>(null)
const uploading = ref(false)
const message = ref<string | null>(null)
const error = ref<string | null>(null)
const dragging = ref(false)
const input = ref<HTMLInputElement | null>(null)
const capeSaving = ref(false)

const usernameInput = ref('')
const loadingUsername = ref(false)

const editingSkinId = ref<string | null>(null)
const editNameInput = ref('')

const masterUrl = useRuntimeConfig().public.masterUrl
const STANDARD_PRESETS = [
  { id: 'steve', name: 'Steve', url: `${masterUrl}/api/textures/presets/steve.png` },
  { id: 'alex', name: 'Alex', url: `${masterUrl}/api/textures/presets/alex.png` },
  { id: 'ari', name: 'Ari', url: `${masterUrl}/api/textures/presets/ari.png` },
  { id: 'zuri', name: 'Zuri', url: `${masterUrl}/api/textures/presets/zuri.png` },
  { id: 'efe', name: 'Efe', url: `${masterUrl}/api/textures/presets/efe.png` },
  { id: 'makena', name: 'Makena', url: `${masterUrl}/api/textures/presets/makena.png` },
  { id: 'kai', name: 'Kai', url: `${masterUrl}/api/textures/presets/kai.png` },
  { id: 'sunny', name: 'Sunny', url: `${masterUrl}/api/textures/presets/sunny.png` },
  { id: 'noor', name: 'Noor', url: `${masterUrl}/api/textures/presets/noor.png` }
]

const { data: capes } = await useAsyncData('user-capes-list', () =>
  auth.request<CapeRow[]>('/api/capes'), { default: () => [] }
)

const currentSkinUrl = computed(() => auth.user.value?.skin_url)
const currentSlim = computed(() => auth.user.value?.skin_slim === true)
const modelSaving = ref(false)

/**
 * Смена модели без перезаливки файла: картинка та же, меняется только ширина
 * рук. Требовать исходник ради галочки нельзя — скин мог приехать по нику.
 */
async function setModel(slim: boolean) {
  if (modelSaving.value || currentSlim.value === slim) return
  modelSaving.value = true
  error.value = null
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/skin/model', {
      method: 'PUT',
      body: { model: slim ? 'slim' : 'classic' },
    })
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    modelSaving.value = false
  }
}
const currentCapeUrl = computed(() => auth.user.value?.cape_url)

const { data: serverPresets, refresh: refreshPresets } = await useAsyncData('user-skin-presets', () =>
  auth.request<SavedSkin[]>('/api/me/skin-presets'), { default: () => [] }
)

const savedSkins = computed(() => serverPresets.value || [])

function accept(next: File | null) {
  error.value = null
  if (!next) return
  if (next.type !== 'image/png') {
    error.value = 'Only PNG files are supported.'
    return
  }
  if (next.size > MAX_BYTES) {
    error.value = `File is too large — ${Math.round(next.size / 1024)} KB of 256 KB allowed.`
    return
  }
  file.value = next
  uploadSkinFile(next)
}

function onPick(event: Event) {
  const el = event.target as HTMLInputElement
  accept(el.files?.[0] || null)
  el.value = ''
}

function onDrop(event: DragEvent) {
  dragging.value = false
  accept(event.dataTransfer?.files?.[0] || null)
}

async function uploadSkinFile(f: File) {
  uploading.value = true
  error.value = null
  try {
    const updated = await auth.upload<UserProfile>('/api/me/skin', 'skin', f)
    auth.user.value = updated
    message.value = 'Skin updated successfully'

    if (updated.skin_url) {
      await auth.request('/api/me/skin-presets', {
        method: 'POST',
        body: {
          name: f.name.replace(/\.png$/i, ''),
          skin_url: updated.skin_url
        }
      })
      await refreshPresets()
    }
  } catch (err) {
    error.value = humanError(err)
    console.error(err)
  } finally {
    uploading.value = false
  }
}

async function uploadSkinByUsername() {
  const name = usernameInput.value.trim()
  if (!name) return
  loadingUsername.value = true
  error.value = null
  message.value = null
  try {
    const updated = await auth.request<UserProfile>('/api/me/skin/from-username', {
      method: 'POST',
      body: { username: name }
    })
    auth.user.value = updated
    message.value = `Skin imported from player '${name}'!`
    usernameInput.value = ''

    if (updated.skin_url) {
      await auth.request('/api/me/skin-presets', {
        method: 'POST',
        body: {
          name: name,
          skin_url: updated.skin_url
        }
      })
      await refreshPresets()
    }
  } catch (err) {
    error.value = humanError(err)
  } finally {
    loadingUsername.value = false
  }
}

async function applyPresetSkin(preset: typeof STANDARD_PRESETS[0]) {
  uploading.value = true
  error.value = null
  try {
    const res = await fetch(preset.url)
    const blob = await res.blob()
    const presetFile = new File([blob], `${preset.name.toLowerCase()}.png`, { type: 'image/png' })
    const updated = await auth.upload<UserProfile>('/api/me/skin', 'skin', presetFile)
    auth.user.value = updated
    message.value = `Equipped preset: ${preset.name}`
  } catch (err) {
    error.value = 'Failed to apply preset skin.'
    console.error(err)
  } finally {
    uploading.value = false
  }
}

async function applySavedSkin(skin: SavedSkin) {
  uploading.value = true
  error.value = null
  try {
    const res = await fetch(skin.skin_url)
    const blob = await res.blob()
    const skinFile = new File([blob], `${skin.name}.png`, { type: 'image/png' })
    // Модель едет вместе с пресетом: без неё сохранённый тонкий скин вернулся
    // бы с толстыми руками, и переключение между пресетами ломало бы вид.
    const updated = await auth.upload<UserProfile>('/api/me/skin', 'skin', skinFile, {
      model: skin.skin_slim ? 'slim' : 'classic',
    })
    auth.user.value = updated
    message.value = `Equipped skin: ${skin.name}`
  } catch (err) {
    error.value = 'Failed to apply skin.'
    console.error(err)
  } finally {
    uploading.value = false
  }
}

function startRename(skin: SavedSkin, ev: Event) {
  ev.stopPropagation()
  editingSkinId.value = skin.id
  editNameInput.value = skin.name
}

async function commitRename(skin: SavedSkin) {
  if (editNameInput.value.trim()) {
    try {
      await auth.request(`/api/me/skin-presets/${skin.id}`, {
        method: 'PUT',
        body: { name: editNameInput.value.trim() }
      })
      await refreshPresets()
    } catch (e) {
      console.error(e)
    }
  }
  editingSkinId.value = null
}

async function deleteSavedSkin(id: string) {
  try {
    await auth.request(`/api/me/skin-presets/${id}`, { method: 'DELETE' })
    await refreshPresets()
  } catch (e) {
    console.error(e)
  }
}

async function resetSkin() {
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/skin', { method: 'DELETE' })
    message.value = 'Skin reset to default'
  } catch (err) {
    error.value = 'Could not reset skin.'
    console.error(err)
  }
}

async function selectCape(capeId: string | null) {
  capeSaving.value = true
  error.value = null
  try {
    auth.user.value = await auth.request<UserProfile>('/api/me/cape', {
      method: 'PUT',
      body: { cape_id: capeId }
    })
    message.value = capeId ? 'Cape equipped!' : 'Cape disabled.'
  } catch (err) {
    error.value = 'Could not update cape.'
    console.error(err)
  } finally {
    capeSaving.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('skin-title')" :subtitle="t('skin-subtitle')">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" :to="link.cabinet()">{{ t('web-nav-cabinet') }}</AtomButton>
    </template>

    <div class="grid gap-6 xl:grid-cols-[340px_1fr] items-start">
      <!-- Left Column: 3D Character Preview -->
      <NoroCard :title="t('skin-3d-character')" icon="i-lucide-user-round">
        <template #actions>
          <UBadge :color="currentSkinUrl ? 'success' : 'neutral'" variant="subtle">
            {{ currentSkinUrl ? t('skin-custom-badge') : t('skin-default-badge') }}
          </UBadge>
        </template>

        <SkinPreview3D :skin-url="currentSkinUrl" :cape-url="currentCapeUrl" :slim="currentSlim" />

        <!-- Сегмент из двух половин, а не две кнопки: выбранная видна фоном,
             и длинная подпись не обрезается — половины делят ширину поровну. -->
        <div v-if="currentSkinUrl" class="pt-3 border-t border-[var(--noro-border)]">
          <div
            class="flex gap-1 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] p-1"
            :title="t('skin-model-hint')"
          >
            <button
              v-for="option in [{ slim: false, label: t('skin-model-classic') }, { slim: true, label: t('skin-model-slim') }]"
              :key="String(option.slim)"
              type="button"
              :disabled="modelSaving"
              class="flex-1 rounded-[var(--noro-r-xs)] px-2 py-1.5 text-[10px] font-bold uppercase tracking-wide transition disabled:opacity-50"
              :class="currentSlim === option.slim
                ? 'bg-[var(--noro-cream)] text-[var(--noro-bg-deep)]'
                : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
              @click="setModel(option.slim)"
            >{{ option.label }}</button>
          </div>
        </div>

        <div v-if="currentSkinUrl" class="pt-2 border-t border-[var(--noro-border)]">
          <AtomButton
            variant="ghost"
            icon="i-lucide-rotate-ccw"
            class="w-full justify-center text-xs text-[var(--noro-danger)]"
            @click="resetSkin"
          >
            {{ t('skin-reset-default') }}
          </AtomButton>
        </div>
      </NoroCard>

      <!-- Right Column: Skins & Capes Manager -->
      <div class="space-y-6">
        <!-- Alerts -->
        <UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="error" />
        <UAlert v-else-if="message" color="success" variant="subtle" icon="i-lucide-check" :description="message" />

        <!-- 1. Saved custom skins and presets -->
        <NoroCard
          :title="t('skin-your-skins')"
          :subtitle="t('skin-drop-hint')"
          icon="i-lucide-bookmark"
        >

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <!-- Add Skin Dropzone Tile Card (+) -->
            <label
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-dashed border-[var(--noro-blue)]/50 bg-[var(--noro-blue)]/5 p-4 text-center transition hover:border-[var(--noro-blue)] hover:bg-[var(--noro-blue)]/10 shadow-sm"
              @dragover.prevent="dragging = true"
              @dragleave.prevent="dragging = false"
              @drop.prevent="onDrop"
            >
              <div class="flex size-12 items-center justify-center rounded-full bg-[var(--noro-blue)]/20 text-[var(--noro-blue)] group-hover:scale-110 transition-transform">
                <UIcon name="i-lucide-plus" class="size-7" />
              </div>
              <span class="mt-3 text-xs font-bold text-[var(--noro-text)]">{{ t('skin-new-skin') }}</span>
              <span class="text-[10px] text-[var(--noro-muted)]">{{ t('skin-upload-png') }}</span>
              <input ref="input" type="file" accept="image/png" class="hidden" @change="onPick">
            </label>

            <!-- Saved Skins Cards -->
            <div
              v-for="skin in savedSkins"
              :key="skin.id"
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-xl border p-3 transition hover:scale-105"
              :class="currentSkinUrl === skin.skin_url
                ? 'border-2 border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-lg'
                : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-cream)]/50'"
              @click="applySavedSkin(skin)"
            >
              <div class="absolute right-1.5 top-1.5 z-10 flex gap-1 opacity-0 group-hover:opacity-100 transition">
                <button
                  type="button"
                  class="size-6 place-items-center rounded bg-black/60 text-white hover:bg-[var(--noro-blue)] flex items-center justify-center"
                  :title="t('web-rules-scope-general')"
                  @click.stop="startRename(skin, $event)"
                >
                  <UIcon name="i-lucide-pencil" class="size-3" />
                </button>
                <button
                  type="button"
                  class="size-6 place-items-center rounded bg-black/60 text-white hover:bg-red-600 flex items-center justify-center"
                  :title="t('web-rules-scope-general')"
                  @click.stop="deleteSavedSkin(skin.id)"
                >
                  <UIcon name="i-lucide-x" class="size-3" />
                </button>
              </div>

              <!-- Name or Rename input -->
              <template v-if="editingSkinId === skin.id">
                <input
                  v-model="editNameInput"
                  type="text"
                  class="w-full rounded border border-[var(--noro-blue)] bg-black px-1.5 py-0.5 text-center text-xs font-bold text-white focus:outline-none"
                  @click.stop
                  @keydown.enter.stop="commitRename(skin)"
                  @blur="commitRename(skin)"
                />
              </template>
              <template v-else>
                <span class="w-full truncate text-center text-xs font-bold text-[var(--noro-text)]">{{ skin.name }}</span>
              </template>

              <SkinCard3D :skin-url="skin.skin_url" :width="100" :height="125" />
              <UBadge v-if="currentSkinUrl === skin.skin_url" color="primary" variant="subtle" class="text-[10px]">{{ t('skin-equipped') }}</UBadge>
              <span v-else class="text-[10px] text-[var(--noro-muted)] group-hover:text-[var(--noro-text)] font-semibold">{{ t('skin-equip') }}</span>
            </div>
          </div>
        </NoroCard>

        <!-- 2. Standard Mojang presets -->
        <NoroCard
          :title="t('skin-official-skins')"
          :subtitle="t('skin-mojang-desc')"
          icon="i-lucide-sparkles"
        >

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <div
              v-for="preset in STANDARD_PRESETS"
              :key="preset.name"
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-xl border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-2.5 transition hover:border-[var(--noro-cream)] hover:scale-105"
              @click="applyPresetSkin(preset)"
            >
              <span class="text-xs font-bold text-[var(--noro-text)]">{{ preset.name }}</span>
              <SkinCard3D :preset="preset.id" />
              <span class="text-[10px] text-[var(--noro-muted)] group-hover:text-[var(--noro-cream)] font-bold">{{ t('skin-equip') }}</span>
            </div>
          </div>
        </NoroCard>

        <!-- 3. Capes grid -->
        <NoroCard
          :title="t('skin-available-capes')"
          :subtitle="t('skin-pick-cape-desc')"
          icon="i-lucide-layers"
        >
          <template #actions>
            <UBadge color="neutral" variant="subtle">{{ t('skin-capes-count', { count: capes.length }) }}</UBadge>
          </template>

          <div v-if="capes.length" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <!-- Option 0: Disable Cape -->
            <div
              class="group relative flex aspect-[10/16] cursor-pointer flex-col items-center justify-center rounded-xl border text-center transition hover:scale-105"
              :class="!currentCapeUrl
                ? 'border-2 border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-lg'
                : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-cream)]/50'"
              @click="selectCape(null)"
            >
              <UIcon name="i-lucide-x" class="size-6 text-[var(--noro-muted)] group-hover:text-[var(--noro-text)]" />
              <span class="mt-1 text-[11px] font-bold text-[var(--noro-muted)]">{{ t('skin-no-cape') }}</span>
            </div>

            <!-- Capes Cards -->
            <div
              v-for="cape in capes"
              :key="cape.id"
              class="group relative flex aspect-[10/16] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-xl border p-2 transition hover:scale-105"
              :class="currentCapeUrl === cape.url
                ? 'border-2 border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-xl'
                : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-cream)]/50'"
              @click="selectCape(cape.id)"
            >
              <span class="w-full truncate text-center text-[11px] font-bold text-[var(--noro-text)]">{{ cape.name }}</span>
              <div class="relative flex size-full items-center justify-center">
                <CapePreview :url="cape.url" :alt="cape.name" class="h-full w-auto max-w-full shadow-md" />
              </div>
            </div>
          </div>

          <EmptyState
            v-else
            icon="i-lucide-shield-off"
            :title="t('skin-no-capes-title')"
            :text="t('skin-no-capes-desc')"
          />
        </NoroCard>
      </div>
    </div>
  </NoroShell>
</template>
