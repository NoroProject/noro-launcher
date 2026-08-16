<script setup lang="ts">
import type { SigningKey } from '~/types/setup'

definePageMeta({ layout: false })

const setup = useSetup()
const notify = useNotify()

const STEPS = ['Token', 'URLs', 'Discord', 'Signing', 'Finish'] as const
const step = ref(0)
const busy = ref(false)
const done = ref(false)
const generated = ref<SigningKey | null>(null)
const envBlock = ref('')

const settings = ref<Record<string, string>>({
  instance_name: '',
  public_url: '',
  web_url: '',
  allowed_origins: '',
  files_cdn_url: '',
  discord_client_id: '',
})

const secretPresent = (name: string) => !!setup.status.value?.secrets?.[name]

async function run(fn: () => Promise<unknown>) {
  busy.value = true
  try {
    await fn()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function useToken(token: string) {
  setup.rememberToken(token)
  // Пустое сохранение — самая дешёвая проверка токена: если он не подошёл,
  // мастер ответит 401, и оператор узнает об этом здесь, а не через три шага.
  await run(async () => {
    await setup.saveSettings({})
    step.value = 1
  })
}

const saveAnd = (next: number) =>
  run(async () => {
    await setup.saveSettings(settings.value)
    step.value = next
  })

const generateKey = () =>
  run(async () => {
    generated.value = await setup.generateSigningKey()
  })

const toFinish = () =>
  run(async () => {
    envBlock.value = (await setup.envBlock()).env
    step.value = 4
  })

const finish = () =>
  run(async () => {
    await setup.complete()
    setup.forgetToken()
    done.value = true
  })

onMounted(async () => {
  const status = await setup.loadStatus().catch(() => null)
  // Настроенный инстанс на визарде делать нечего: токена нет, а страница
  // обещала бы то, чего уже не произойдёт.
  if (status?.setup_completed) await navigateTo('/')
  if (setup.token.value) step.value = 1
})
</script>

<template>
  <div class="min-h-screen bg-[var(--noro-bg-deep)] px-4 py-12">
    <div class="mx-auto grid w-full max-w-3xl gap-6">
      <header>
        <h1 class="noro-pixel text-3xl uppercase text-[var(--noro-cream)]">Instance setup</h1>
        <p class="mt-2 text-sm text-[var(--noro-muted)]">
          Two phases, because secrets only ever live in the environment: this
          wizard writes what it can to the database and hands you the rest for
          your <code>.env</code>.
        </p>
      </header>

      <ol class="flex flex-wrap gap-2">
        <li
          v-for="(name, i) in STEPS"
          :key="name"
          class="rounded px-3 py-1.5 text-xs font-black uppercase tracking-wider"
          :class="i === step
            ? 'bg-[var(--noro-magenta)] text-[var(--noro-white)]'
            : i < step
              ? 'bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] text-[var(--noro-cream)]'
              : 'bg-[var(--noro-input)] text-[var(--noro-muted)]'"
        >{{ i + 1 }}. {{ name }}</li>
      </ol>

      <section class="noro-panel p-6">
        <SetupStepToken v-if="step === 0" @done="useToken" />

        <SetupStepUrls v-else-if="step === 1" v-model="settings" @next="saveAnd(2)" />

        <SetupStepDiscord
          v-else-if="step === 2"
          v-model="settings"
          :api-url="settings.public_url"
          :secret-present="secretPresent('DISCORD_CLIENT_SECRET')"
          @next="saveAnd(3)"
        />

        <SetupStepSigning
          v-else-if="step === 3"
          v-model="generated"
          :key-present="secretPresent('NORO_SIGNING_KEY')"
          @generate="generateKey"
          @next="toFinish"
        />

        <SetupStepFinish
          v-else
          :env-block="envBlock"
          :secure-context="setup.status.value?.secure_context ?? false"
          :done="done"
          @finish="finish"
        />
      </section>

      <p v-if="busy" class="text-xs text-[var(--noro-muted)]">Working…</p>
    </div>
  </div>
</template>
