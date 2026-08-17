<script setup lang="ts">
import type { AgentFile } from '~/types/agent'

const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const host = useRequestURL().host

const { data: files } = await useAsyncData('admin-agents', () =>
  auth.request<AgentFile[]>('/api/admin/agents'), { default: () => [] as AgentFile[] }
)

const wrapper = computed(() => files.value.find(file => file.platform === 'wrapper') || null)
</script>

<template>
  <NoroShell :title="t('admin-wrapper-title')" :subtitle="t('admin-wrapper-lead')">
    <div class="grid gap-5 items-start">
      <AdminWrapperSetup :wrapper="wrapper" :host="host" />
      <AdminWrapperOptions />
      <AdminAgentDownloads :agents="files" />
    </div>
  </NoroShell>
</template>
