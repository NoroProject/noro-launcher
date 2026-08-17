<script setup lang="ts">
/** Свод правил проекта. Открыт всем — вход не нужен. */
definePageMeta({ layout: 'public' })

const { t } = useT()
const { serverId, search, scopes, tree, loose, found, rules, sanctions, pending, failed } = useRules()

const empty = computed(() => !pending.value && found.value === 0)
const total = computed(() => rules.value.length)

useHead({
  title: () => t('web-rules-meta-title'),
  meta: [{ name: 'description', content: () => t('web-rules-meta-description') }],
})
</script>

<template>
  <div class="mx-auto max-w-7xl px-5 py-10">
    <section class="flex flex-wrap items-end justify-between gap-4 border-b border-[var(--noro-border)] pb-8">
      <div>
        <h1 class="noro-pixel text-3xl uppercase text-[var(--noro-cream)] md:text-4xl">
          {{ t('web-rules-title') }}
        </h1>
        <p class="mt-3 max-w-3xl text-base font-medium leading-7 text-[var(--noro-text)]">
          {{ t('web-rules-lead') }}
        </p>
      </div>
      <div class="text-right">
        <div class="noro-label">{{ t('web-rules-total') }}</div>
        <div class="noro-pixel text-2xl text-[var(--noro-cream)]">{{ total }}</div>
      </div>
    </section>

    <!-- Поиск и выбор свода. Липкая панель без отрицательных полей: они
         делали её шире экрана, и на телефоне страница ездила вбок. -->
    <section class="sticky top-[72px] z-20 py-4">
      <div class="noro-panel flex flex-col gap-3 bg-[var(--noro-panel)] p-4 lg:flex-row lg:items-center">
        <label class="relative flex-1">
          <UIcon
            name="i-lucide-search"
            class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
          />
          <input
            v-model="search"
            class="noro-input w-full !pl-10"
            :placeholder="t('web-rules-search')"
            :aria-label="t('web-rules-search-aria')"
          >
        </label>

        <div v-if="scopes.length" class="flex flex-wrap items-center gap-2">
          <span class="noro-label">{{ t('web-rules-scope') }}</span>
          <button
            type="button"
            class="noro-chip px-3 py-2 text-xs font-bold"
            :class="serverId ? 'text-[var(--noro-muted)]' : 'noro-chip-on'"
            @click="serverId = ''"
          >{{ t('web-rules-scope-general') }}</button>
          <button
            v-for="scope in scopes"
            :key="scope.id"
            type="button"
            class="noro-chip px-3 py-2 text-xs font-bold"
            :class="serverId === scope.id ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
            @click="serverId = scope.id"
          >{{ scope.name }}</button>
        </div>
      </div>
      <p v-if="search" class="mt-2 text-xs font-bold text-[var(--noro-muted)]">
        {{ t('web-rules-found', { count: found, query: search }) }}
      </p>
    </section>

    <div class="grid gap-8 lg:grid-cols-[260px_1fr]">
      <aside class="hidden lg:block">
        <RulesToc v-if="tree.length" :nodes="tree" :loose="loose.length" />
      </aside>

      <div class="min-w-0">
        <div v-if="pending" class="noro-panel grid place-items-center gap-2 p-12 text-[var(--noro-muted)]">
          <UIcon name="i-lucide-loader-2" class="size-6 animate-spin" />
          <span class="text-sm font-bold">{{ t('web-rules-loading') }}</span>
        </div>

        <EmptyState
          v-else-if="failed"
          icon="i-lucide-plug-zap"
          :title="t('web-rules-failed-title')"
          :text="t('web-rules-failed-text')"
        />

        <EmptyState
          v-else-if="empty && search"
          icon="i-lucide-search-x"
          :title="t('web-rules-nomatch-title')"
          :text="t('web-rules-nomatch-text', { query: search })"
        />

        <EmptyState
          v-else-if="empty"
          icon="i-lucide-book-open"
          :title="t('web-rules-empty-title')"
          :text="t('web-rules-empty-text')"
        />

        <div v-else class="grid gap-10">
          <RulesSection v-for="node in tree" :key="node.category.id" :node="node" :sanctions="sanctions" />

          <section v-if="loose.length" id="section-loose" class="scroll-mt-24">
            <header class="border-b border-[var(--noro-border)] pb-3">
              <h2 class="text-2xl font-black text-[var(--noro-cream)]">{{ t('web-rules-other') }}</h2>
            </header>
            <div class="mt-4 grid gap-3">
              <RulesCard
                v-for="rule in loose"
                :key="rule.id"
                :rule="rule"
                :sanctions="sanctions.filter(s => s.rule_id === rule.id)"
              />
            </div>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>
