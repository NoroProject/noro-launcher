<script setup lang="ts">
import type { NewsRow } from '~/types/api'

const route = useRoute()
const auth = useAuth()
await auth.loadMe()

const id = computed(() => String(route.params.id))
const { data: news, refresh } = await useAsyncData('admin-news-edit-list', () =>
  auth.request<NewsRow[]>('/api/admin/news'), { default: () => [] }
)
const item = computed(() => news.value.find(row => row.id === id.value))
const form = reactive({ title: '', body: '', preview_img_url: '', pinned: false })
const busy = ref<string | null>(null)

watchEffect(() => {
  if (!item.value) return
  Object.assign(form, {
    title: item.value.title,
    body: item.value.body,
    preview_img_url: item.value.preview_img_url || '',
    pinned: item.value.pinned
  })
})

async function save() {
  busy.value = 'save'
  try {
    await auth.request(`/api/admin/news/${id.value}`, {
      method: 'PUT',
      body: { ...form, preview_img_url: form.preview_img_url || null }
    })
    await refresh()
  } finally {
    busy.value = null
  }
}

async function removeNews() {
  busy.value = 'delete'
  try {
    await auth.request(`/api/admin/news/${id.value}`, { method: 'DELETE' })
    await navigateTo('/admin/news')
  } finally {
    busy.value = null
  }
}
</script>

<template>
  <NoroShell :title="item?.title || 'NEWS'" subtitle="Markdown editor">
    <template #actions>
      <NuxtLink :to="'/admin/news'" class="noro-btn noro-btn-ghost">
        <UIcon name="i-lucide-arrow-left" class="size-4" />
        Back
      </NuxtLink>
    </template>

    <EmptyState v-if="!item" icon="i-lucide-search-x" title="News item not found" />

    <div v-else class="grid gap-5 xl:grid-cols-[1fr_420px]">
      <form class="noro-panel grid gap-3 p-5" @submit.prevent="save">
        <label><span class="noro-label">Title</span><input v-model="form.title" class="noro-input" required></label>
        <AdminImagePicker v-model="form.preview_img_url" label="Preview image" />
        <div class="grid gap-2"><span class="noro-label">Body</span><AdminMarkdownEditor v-model="form.body" /></div>
        <UCheckbox v-model="form.pinned" label="Pinned" />
        <div class="flex gap-2">
          <button type="submit" class="noro-btn noro-btn-primary" :disabled="busy === 'save'">
            <UIcon
              :name="busy === 'save' ? 'i-lucide-loader-circle' : 'i-lucide-save'"
              class="size-4"
              :class="busy === 'save' ? 'animate-spin' : ''"
            />
            Save
          </button>
          <UButton :loading="busy === 'delete'" icon="i-lucide-trash-2" color="error" variant="subtle" @click="removeNews">Delete</UButton>
        </div>
      </form>

      <section class="noro-panel p-5">
        <h2 class="mb-4 font-bold text-[var(--noro-text)]">Preview</h2>
        
        <h3 class="text-xl font-black text-[var(--noro-text)]">{{ form.title }}</h3>
        <pre class="mt-3 whitespace-pre-wrap rounded-lg bg-[var(--noro-input)] p-3 text-sm text-[var(--noro-text)]">{{ form.body }}</pre>
      </section>
    </div>
  </NoroShell>
</template>
