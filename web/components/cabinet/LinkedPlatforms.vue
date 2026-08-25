<script setup lang="ts">
/**
 * Привязанные платформы игрока: список, привязка ещё одной, отвязка.
 *
 * Платформу регистрации отвязать нельзя — из неё выведен Minecraft-UUID
 * игрока, а с ним его инвентарь, прогресс и права на всех серверах.
 */

const props = defineProps<{ providers: AuthProviderInfo[] }>()

const notify = useNotify()
const { t } = useT()
const { identities, startLink, unlink } = useIdentities()

/** Показываем только то, что можно привязать: включено и ещё не привязано. */
const linkable = computed(() =>
  props.providers.filter((p) => !identities.value.some((i) => i.provider === p.provider))
)

async function link(provider: string) {
  try {
    await startLink(provider)
  } catch (e) {
    notify.fail(e)
  }
}

async function remove(provider: string) {
  try {
    await unlink(provider)
    notify.ok()
  } catch (e) {
    notify.fail(e)
  }
}

// Привязка возвращает игрока сюда — с результатом в адресной строке.
const route = useRoute()
onMounted(() => {
  if (route.query.linked) notify.ok()
  if (route.query.link_error === 'taken') {
    notify.fail(null, t('cabinet-identity-taken', { provider: String(route.query.provider || '') }))
  }
})
</script>

<template>
  <NoroCard
    :title="t('cabinet-identities-title')"
    :subtitle="t('cabinet-identities-lead')"
    icon="i-lucide-link"
  >
    <div class="grid gap-2">
      <div
        v-for="i in identities"
        :key="i.provider"
        class="flex items-center justify-between gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3"
      >
        <div class="flex min-w-0 items-center gap-3">
          <UIcon
            :name="providerMeta(i.provider).icon"
            class="size-5 shrink-0"
            :style="providerIconStyle(i.provider)"
          />
          <div class="min-w-0">
            <div class="truncate text-sm font-bold text-[var(--noro-text)]">{{ providerMeta(i.provider).label }}</div>
            <div class="truncate text-xs text-[var(--noro-muted)]">{{ i.username || i.provider_user_id }}</div>
          </div>
        </div>
        <span
          v-if="i.is_primary"
          class="shrink-0 text-[10px] font-bold uppercase tracking-wider text-[var(--noro-muted)]"
        >
          {{ t('cabinet-identity-primary') }}
        </span>
        <AtomButton v-else variant="danger" size="sm" icon="i-lucide-unlink" @click="remove(i.provider)">
          {{ t('cabinet-identity-unlink') }}
        </AtomButton>
      </div>

      <EmptyState
        v-if="!identities.length"
        :title="t('cabinet-identities-none-title')"
        :text="t('cabinet-identities-none-text')"
      />

      <!-- Привязывать нечего — говорим об этом. Пустое место под списком читается
           как «кнопку не нарисовали», хотя дело в том, что других способов входа
           на инстансе просто нет: платформа предлагается, только если оператор
           её включил и задал ключи приложения. -->
      <p v-if="!linkable.length" class="pt-1 text-xs text-[var(--noro-muted)]">
        {{ t('cabinet-identity-nothing-to-link') }}
      </p>

      <div v-else class="flex flex-wrap gap-2 pt-1">
        <!-- Значок здесь нейтральный, а не логотип платформы: кнопка красит
             свою иконку цветом текста, а марку Discord или Twitch чужим
             цветом показывать нельзя. Логотип — в строке привязки выше, где
             цвет задаём мы. -->
        <AtomButton
          v-for="p in linkable"
          :key="p.provider"
          variant="secondary"
          size="sm"
          icon="i-lucide-plus"
          @click="link(p.provider)"
        >
          {{ t('cabinet-identity-link', { provider: p.name }) }}
        </AtomButton>
      </div>
    </div>
  </NoroCard>
</template>
