<script setup lang="ts">
import type { NewsRow } from '~/types/api'

const auth = useAuth()
await auth.loadMe()

const { data: news, refresh, pending } = await useAsyncData('admin-news', () =>
  auth.request<NewsRow[]>('/api/admin/news'), { default: () => [] }
)

const form = reactive({ title: '', body: '', preview_img_url: '', pinned: false })
const creating = ref(false)
const showCreate = ref(false)

async function createNews() {
  creating.value = true
  try {
    await auth.request('/api/admin/news', {
      method: 'POST',
      body: { ...form, preview_img_url: form.preview_img_url || null }
    })
    Object.assign(form, { title: '', body: '', preview_img_url: '', pinned: false })
    await refresh()
    showCreate.value = false
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <NoroShell title="NEWS" subtitle="Markdown posts for launcher">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
      <button type="button" class="noro-btn noro-btn-primary" @click="showCreate = true">
        <UIcon name="i-lucide-plus" class="size-5" />New post
      </button>
    </template>

    <section class="grid gap-3">
      <NuxtLink
        v-for="item in news"
        :key="item.id"
        :to="`/admin/news/${item.id}`"
        class="noro-panel block p-4 transition hover:border-[var(--noro-magenta)]"
      >
        <div class="flex items-start justify-between gap-3">
          <div>
            <h2 class="font-bold text-[var(--noro-text)]">{{ item.title }}</h2>
            <p class="mt-1 line-clamp-2 text-sm text-[var(--noro-muted)]">{{ item.body }}</p>
          </div>
          <UBadge v-if="item.pinned" color="warning" variant="subtle">pinned</UBadge>
        </div>
      </NuxtLink>
      <EmptyState v-if="!news?.length" icon="i-lucide-newspaper" title="No news yet" text="Create the first post from the toolbar." />
    </section>

    <AtomModal v-model="showCreate" title="NEW POST" subtitle="Publish launcher news in Markdown" wide>
      <form class="grid gap-3" @submit.prevent="createNews">
        <label><span class="noro-label">Title</span><input v-model="form.title" class="noro-input" required></label>
        <AdminImagePicker v-model="form.preview_img_url" label="Preview image" />
        <div class="grid gap-2"><span class="noro-label">Body</span><AdminMarkdownEditor v-model="form.body" /></div>
        <UCheckbox v-model="form.pinned" label="Pinned" />
        <div class="flex justify-end gap-3 pt-2">
          <button type="button" class="noro-btn noro-btn-secondary" @click="showCreate = false">Cancel</button>
          <button :disabled="creating" type="submit" class="noro-btn noro-btn-primary">
            <UIcon name="i-lucide-plus" class="size-5" />Create
          </button>
        </div>
      </form>
    </AtomModal>
  </NoroShell>
</template>
