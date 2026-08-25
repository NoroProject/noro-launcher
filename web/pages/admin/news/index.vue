<script setup lang="ts">
import type { NewsRow } from '~/types/api'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)

const notify = useNotify()
await auth.loadMe()

const list = usePagedList<NewsRow>('admin-news', '/api/admin/news')
const news = list.items
const pending = list.pending
const refresh = list.refresh

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
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('admin-news-title')" :subtitle="t('admin-news-subtitle')">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
      <AtomButton v-if="can('noro.admin.news.edit')" variant="primary" icon="i-lucide-plus" @click="showCreate = true">{{ t('admin-news-new-post') }}</AtomButton>
    </template>

    <div class="noro-panel mb-4 p-4">
      <label class="noro-label mb-2 block">{{ t('admin-users-search-label') }}</label>
      <input v-model="list.search.value" class="noro-input w-full" :placeholder="t('admin-news-search-placeholder')">
    </div>

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
          <UBadge v-if="item.pinned" color="warning" variant="subtle">{{ t('admin-news-pinned') }}</UBadge>
        </div>
      </NuxtLink>
      <EmptyState
        v-if="!news?.length"
        icon="i-lucide-newspaper"
        :title="list.search.value ? t('paging-empty') : t('admin-news-empty-title')"
        :text="list.search.value ? '' : t('admin-news-empty-text')"
      />

      <NoroPager
        :page="list.page.value"
        :pages="list.pages.value"
        :total="list.total.value"
        :per-page="list.perPage"
        @go="list.goTo"
      />
    </section>

    <AtomModal v-model="showCreate" :title="t('admin-news-modal-title')" :subtitle="t('admin-news-modal-subtitle')" wide>
      <form class="grid gap-3" @submit.prevent="createNews">
        <label><span class="noro-label">{{ t('admin-news-post-title') }}</span><input v-model="form.title" class="noro-input" required></label>
        <AdminImagePicker v-model="form.preview_img_url" label="Preview image" />
        <div class="grid gap-2"><span class="noro-label">{{ t('admin-news-post-body') }}</span><AdminMarkdownEditor v-model="form.body" /></div>
        <UCheckbox v-model="form.pinned" :label="t('admin-news-post-pinned')" />
        <div class="flex justify-end gap-3 pt-2">
          <AtomButton variant="secondary" @click="showCreate = false">{{ t('web-rules-cancel') }}</AtomButton>
          <AtomButton variant="primary" icon="i-lucide-plus" :disabled="creating" type="submit">{{ t('admin-notes-add') }}</AtomButton>
        </div>
      </form>
    </AtomModal>
  </NoroShell>
</template>
