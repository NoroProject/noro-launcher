<script setup lang="ts">
import type { UserProfile } from '~/types/api'
import type { CapeRow } from '~/types/cape'

const auth = useAuth()
await auth.loadMe()

const MAX_BYTES = 256 * 1024

interface SavedSkin {
  id: string
  name: string
  url: string
}

const file = ref<File | null>(null)
const uploading = ref(false)
const message = ref<string | null>(null)
const error = ref<string | null>(null)
const dragging = ref(false)
const input = ref<HTMLInputElement | null>(null)
const capeSaving = ref(false)

const savedSkins = ref<SavedSkin[]>([])

// Default official Minecraft standard presets
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
const currentCapeUrl = computed(() => auth.user.value?.cape_url)

onMounted(() => {
  try {
    const raw = localStorage.getItem('noro_saved_skins')
    if (raw) savedSkins.value = JSON.parse(raw)
  } catch (e) {
    console.error(e)
  }
})

function saveSkinsToStorage() {
  try {
    localStorage.setItem('noro_saved_skins', JSON.stringify(savedSkins.value))
  } catch (e) {
    console.error(e)
  }
}

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
  // Иначе повторный выбор того же файла не даст события change, и попытка
  // перезалить скин после ошибки выглядит как «ничего не происходит».
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
      const exists = savedSkins.value.some(s => s.url === updated.skin_url)
      if (!exists) {
        savedSkins.value.unshift({
          id: String(Date.now()),
          name: f.name.replace(/\.png$/i, ''),
          url: updated.skin_url
        })
        saveSkinsToStorage()
      }
    }
  } catch (err) {
    error.value = humanError(err)
    console.error(err)
  } finally {
    uploading.value = false
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
    const res = await fetch(skin.url)
    const blob = await res.blob()
    const skinFile = new File([blob], `${skin.name}.png`, { type: 'image/png' })
    const updated = await auth.upload<UserProfile>('/api/me/skin', 'skin', skinFile)
    auth.user.value = updated
    message.value = `Equipped skin: ${skin.name}`
  } catch (err) {
    error.value = 'Failed to apply skin.'
    console.error(err)
  } finally {
    uploading.value = false
  }
}

function deleteSavedSkin(id: string) {
  savedSkins.value = savedSkins.value.filter(s => s.id !== id)
  saveSkinsToStorage()
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
  <NoroShell title="SKINS & CAPES" subtitle="Modrinth & Pandora style skin manager">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" to="/cabinet">Cabinet</AtomButton>
    </template>

    <div class="grid gap-6 xl:grid-cols-[340px_1fr] items-start">
      <!-- Left Column: 3D Character Preview -->
      <section class="noro-panel bg-[var(--noro-bg-deep)] p-5 space-y-4">
        <div class="flex items-center justify-between">
          <h2 class="noro-label">3D Character</h2>
          <UBadge :color="currentSkinUrl ? 'success' : 'neutral'" variant="subtle">
            {{ currentSkinUrl ? 'Custom Skin' : 'Default' }}
          </UBadge>
        </div>

        <SkinPreview3D :skin-url="currentSkinUrl" :cape-url="currentCapeUrl" />

        <div class="flex flex-wrap gap-2 pt-2 border-t border-[var(--noro-border)]">
          <AtomButton
            v-if="currentSkinUrl"
            variant="ghost"
            icon="i-lucide-rotate-ccw"
            class="w-full justify-center text-xs text-[var(--noro-danger)]"
            @click="resetSkin"
          >
            Reset to Default
          </AtomButton>
        </div>
      </section>

      <!-- Right Column: Skins & Capes Manager -->
      <div class="space-y-6">
        <!-- Alerts -->
        <UAlert v-if="error" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="error" />
        <UAlert v-else-if="message" color="success" variant="subtle" icon="i-lucide-check" :description="message" />

        <!-- 1. Saved Custom Skins (Сохранённые скины) -->
        <section class="noro-panel p-5 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
                <UIcon name="i-lucide-bookmark" class="size-4 text-[var(--noro-cream)]" />
                Сохранённые скины
              </h2>
              <p class="text-xs text-[var(--noro-muted)]">Загружайте свои скины или выбирайте из вашей коллекции</p>
            </div>
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <!-- Add Skin Dropzone Card -->
            <label
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-center rounded-lg border-2 border-dashed border-[var(--noro-border)] bg-[var(--noro-input)] p-3 text-center transition hover:border-[var(--noro-cream)] hover:bg-[var(--noro-panel)]"
              @dragover.prevent="dragging = true"
              @dragleave.prevent="dragging = false"
              @drop.prevent="onDrop"
            >
              <UIcon name="i-lucide-plus" class="size-8 text-[var(--noro-muted)] group-hover:text-[var(--noro-cream)] group-hover:scale-110 transition-transform" />
              <span class="mt-2 text-xs font-bold text-[var(--noro-text)]">Добавить скин</span>
              <span class="text-[10px] text-[var(--noro-muted)]">Перетащите сюда</span>
              <!-- Инпут обязан лежать внутри label: без этого клик по зоне не
                   открывает диалог, и «загрузка не работает». -->
              <input ref="input" type="file" accept="image/png" class="hidden" @change="onPick">
            </label>

            <!-- Saved Skins Cards -->
            <div
              v-for="skin in savedSkins"
              :key="skin.id"
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-lg border p-2.5 transition hover:scale-105"
              :class="currentSkinUrl === skin.url
                ? 'border-2 border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-lg'
                : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-cream)]/50'"
              @click="applySavedSkin(skin)"
            >
              <button
                type="button"
                class="absolute right-1.5 top-1.5 z-10 hidden size-6 place-items-center rounded bg-black/60 text-white hover:bg-red-600 group-hover:grid"
                @click.stop="deleteSavedSkin(skin.id)"
              >
                <UIcon name="i-lucide-x" class="size-3.5" />
              </button>

              <span class="w-full truncate text-center text-xs font-bold text-[var(--noro-text)]">{{ skin.name }}</span>
              <SkinCard3D :skin-url="skin.url" :width="100" :height="125" />
              <UBadge v-if="currentSkinUrl === skin.url" color="primary" variant="subtle" class="text-[10px]">Активен</UBadge>
              <span v-else class="text-[10px] text-[var(--noro-muted)] group-hover:text-[var(--noro-text)]">Выбрать</span>
            </div>
          </div>
        </section>

        <!-- 2. Standard Presets (Стандартные скины) -->
        <section class="noro-panel p-5 space-y-4">
          <div>
            <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-sparkles" class="size-4 text-[var(--noro-cream)]" />
              Стандартные скины Minecraft
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">Официальные скины персонажей Minecraft</p>
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <div
              v-for="preset in STANDARD_PRESETS"
              :key="preset.name"
              class="group relative flex aspect-[3/4] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-2.5 transition hover:border-[var(--noro-cream)] hover:scale-105"
              @click="applyPresetSkin(preset)"
            >
              <span class="text-xs font-bold text-[var(--noro-text)]">{{ preset.name }}</span>
              <SkinCard3D :preset="preset.id" />
              <span class="text-[10px] text-[var(--noro-muted)] group-hover:text-[var(--noro-cream)] font-bold">Надеть</span>
            </div>
          </div>
        </section>

        <!-- 3. Capes Grid (Плащи) -->
        <section class="noro-panel p-5 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
                <UIcon name="i-lucide-layers" class="size-4 text-[var(--noro-cream)]" />
                Ваши доступные плащи
              </h2>
              <p class="text-xs text-[var(--noro-muted)]">Выберите плащ для вашего персонажа</p>
            </div>
            <UBadge color="neutral" variant="subtle">{{ capes.length }} плащей</UBadge>
          </div>

          <div v-if="capes.length" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            <!-- Option 0: Disable Cape -->
            <div
              class="group relative flex aspect-[10/16] cursor-pointer flex-col items-center justify-center rounded-lg border text-center transition hover:scale-105"
              :class="!currentCapeUrl
                ? 'border-2 border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-lg'
                : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-cream)]/50'"
              @click="selectCape(null)"
            >
              <UIcon name="i-lucide-x" class="size-6 text-[var(--noro-muted)] group-hover:text-[var(--noro-text)]" />
              <span class="mt-1 text-[11px] font-bold text-[var(--noro-muted)]">Без плаща</span>
            </div>

            <!-- Capes Cards -->
            <div
              v-for="cape in capes"
              :key="cape.id"
              class="group relative flex aspect-[10/16] cursor-pointer flex-col items-center justify-between overflow-hidden rounded-lg border p-2 transition hover:scale-105"
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
            title="Нет доступных плащей"
            description="У вас пока нет доступных плащей. Обратитесь к администратору для выдачи прав на плащи!"
          />
        </section>
      </div>
    </div>
  </NoroShell>
</template>
