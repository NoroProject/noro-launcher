<script setup lang="ts">
import type { AgentFile } from '~/types/agent'

const props = defineProps<{ wrapper: AgentFile | null; host: string }>()
const { t } = useT()

const masterUrl = computed(() => `https://${props.host}`)
</script>

<template>
  <section class="noro-panel p-5">
    <div class="mb-4 flex items-center gap-3">
      <div class="grid size-10 place-items-center rounded bg-[var(--noro-blue)]/10 text-[var(--noro-blue)]">
        <UIcon name="i-lucide-terminal" class="size-5" />
      </div>
      <h3 class="text-lg font-black text-white">{{ t('admin-wrapper-setup-title') }}</h3>
    </div>

    <div class="grid gap-5 text-[var(--noro-muted)]">
      <p>
        {{ t('admin-wrapper-setup-intro') }}
      </p>

      <div class="grid gap-5">
        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">{{ t('admin-wrapper-setup-step1') }}</h4>
          <p class="mb-3 text-sm">{{ t('admin-wrapper-setup-step1-hint') }}</p>
          <AtomButton
            v-if="wrapper"
            variant="primary"
            icon="i-lucide-download"
            :href="wrapper.url"
            download="wrapper.jar"
            class="inline-flex w-auto"
          >
            {{ t('admin-wrapper-download-btn') }}
          </AtomButton>
          <p v-else class="text-sm text-[var(--noro-amber)]">
            {{ t('admin-wrapper-setup-not-built') }}
          </p>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">{{ t('admin-wrapper-setup-step2') }}</h4>
          <p class="mb-2 text-sm">
            {{ t('admin-wrapper-setup-step2-desc') }}
          </p>
          <pre class="overflow-x-auto rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs text-[var(--noro-cream)]">master-url={{ masterUrl }}
secret=noroagent_...
server-jar=paper-1.21.1.jar
jvm-args=-Xmx4G -Xms4G</pre>
          <p class="mt-2 text-sm">
            {{ t('admin-wrapper-setup-step2-min') }}
          </p>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">{{ t('admin-wrapper-setup-step3') }}</h4>
          <p class="mb-2 text-sm">
            {{ t('admin-wrapper-setup-step3-desc') }}
          </p>
          <div class="rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs break-all text-[var(--noro-cream)]">
            java -jar wrapper.jar
          </div>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">{{ t('admin-wrapper-setup-step4') }}</h4>
          <p class="text-sm">
            {{ t('admin-wrapper-setup-step4-desc') }}
          </p>
          <div class="mt-2 rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs text-[var(--noro-cream)]">
            online-mode=true
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
