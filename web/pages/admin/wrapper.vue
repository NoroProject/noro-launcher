<script setup lang="ts">
import type { AgentFile } from '~/types/agent'

const auth = useAuth()
await auth.loadMe()

const host = useRequestURL().host

const { data: files } = await useAsyncData('admin-agents', () =>
  auth.request<AgentFile[]>('/api/admin/agents'), { default: () => [] as AgentFile[] }
)

const wrapper = computed(() => files.value.find(file => file.platform === 'wrapper') || null)
</script>

<template>
  <NoroShell title="SERVER WRAPPER" subtitle="Agent installer and supervisor for game servers">
    <div class="grid gap-5 items-start">
      <AdminWrapperSetup :wrapper="wrapper" :host="host" />
      <AdminWrapperOptions />
      <AdminAgentDownloads :agents="files" />
    </div>
  </NoroShell>
</template>
