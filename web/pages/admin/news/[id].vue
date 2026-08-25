<script setup lang="ts">
import type { NewsRow } from '~/types/api'

const route = useRoute()
const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const id = computed(() => String(route.params.id))
// Одну запись, а не весь список: со списочной пагинацией запись со второй
// страницы в выдачу не попадала и форма открывалась пустой.
const { data: item, refresh } = await useAsyncData(
  () => `admin-news-${id.value}`,
  () => auth.request<NewsRow>(`/api/admin/news/${id.value}`),
)
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
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = null
  }
}

async function removeNews() {
  busy.value = 'delete'
  try {
    await auth.request(`/api/admin/news/${id.value}`, { method: 'DELETE' })
    await navigateTo(adminLink.news())
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = null
  }
}
</script>

<template>
  <NoroShell :title="item?.title || t('admin-news-title')" subtitle="Markdown editor">
    <template #actions>
      <AtomButton variant="ghost" icon="i-lucide-arrow-left" :to="'/admin/news'">{{ t('admin-role-back') }}</AtomButton>
    </template>

    <EmptyState v-if="!item" icon="i-lucide-search-x" :title="t('admin-news-empty-title')" />

    <div v-else class="grid gap-5 xl:grid-cols-[1fr_420px]">
      <form class="noro-panel grid gap-3 p-5" @submit.prevent="save">
        <label><span class="noro-label">{{ t('admin-news-post-title') }}</span><input v-model="form.title" class="noro-input" required></label>
        <AdminImagePicker v-model="form.preview_img_url" label="Preview image" />
        <div class="grid gap-2"><span class="noro-label">{{ t('admin-news-post-body') }}</span><AdminMarkdownEditor v-model="form.body" /></div>
        <UCheckbox v-model="form.pinned" :label="t('admin-news-post-pinned')" />
        <div class="flex gap-2">
          <AtomButton
            v-if="can('noro.admin.news.edit')"
            variant="primary"
            icon="i-lucide-save"
            type="submit"
            :loading="busy === 'save'"
          >
            {{ t('cabinet-save') }}
          </AtomButton>
          <AtomButton v-if="can('noro.admin.news.delete')" variant="danger" :loading="busy === 'delete'" icon="i-lucide-trash-2" @click="removeNews">{{ t('admin-blocklist-act-delete') }}</AtomButton>
        </div>
      </form>

      <section class="noro-panel p-5">
        <h2 class="mb-4 font-bold text-[var(--noro-text)]">{{ t('admin-news-preview') }}</h2>
        
        <h3 class="text-xl font-black text-[var(--noro-text)]">{{ form.title }}</h3>
        <pre class="mt-3 whitespace-pre-wrap rounded-lg bg-[var(--noro-input)] p-3 text-sm text-[var(--noro-text)]">{{ form.body }}</pre>
      </section>
    </div>
  </NoroShell>
</template>
