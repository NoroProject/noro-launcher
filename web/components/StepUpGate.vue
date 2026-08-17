<script setup lang="ts">
/**
 * Повторное подтверждение личности перед опасным действием.
 *
 * Ключ доступа — основной путь. Код восстановления остаётся запасным: на
 * инстансе по `http://` без домена WebAuthn не работает вовсе, и подтвердиться
 * там больше нечем. Успех открывает окно на несколько минут, поэтому коды не
 * сгорают по одному на каждое действие.
 */
interface StepUpStatus {
  active: boolean
  passkey_available: boolean
  recovery_codes_left: number
  window_minutes: number
}

const emit = defineEmits<{ confirmed: [] }>()

const auth = useAuth()
const { t } = useT()
const notify = useNotify()

const status = ref<StepUpStatus | null>(null)
const useCode = ref(false)
const code = ref('')
const busy = ref(false)

async function load() {
  try {
    status.value = await auth.request<StepUpStatus>('/api/admin/step-up')
    useCode.value = !status.value.passkey_available
  } catch {
    // Статус не пришёл — показываем поле кода: оно работает всегда.
    useCode.value = true
  }
}

function done() {
  notify.ok(`Confirmed for the next ${status.value?.window_minutes ?? 15} minutes`)
  code.value = ''
  emit('confirmed')
}

async function confirmPasskey() {
  busy.value = true
  try {
    const opts = await auth.request<ChallengeRes>('/api/auth/passkeys/login/options', {
      method: 'POST',
    })
    const credential = await getCredential(opts)
    await auth.request('/api/admin/step-up/passkey', {
      method: 'POST',
      body: { state_id: opts.state_id, credential },
    })
    done()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function confirmCode() {
  busy.value = true
  try {
    await auth.request('/api/admin/step-up/recovery', {
      method: 'POST',
      body: { code: code.value.trim() },
    })
    done()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

onMounted(() => load())
</script>

<template>
  <div class="grid gap-3">
    <p class="text-xs text-[var(--noro-muted)]">
      {{ t('admin-users-impersonate-stepup', { mins: status?.window_minutes ?? 15 }) }}
    </p>

    <template v-if="useCode">
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-users-impersonate-code') }}</span>
        <input v-model="code" class="noro-input w-full font-mono" placeholder="XXXX-XXXX-XXXX">
      </label>
      <AtomButton icon="i-lucide-shield-check" :loading="busy" :disabled="!code.trim()" @click="confirmCode">
        {{ t('admin-users-impersonate-confirm') }}
      </AtomButton>
    </template>

    <AtomButton v-else variant="primary" icon="i-lucide-key-round" :loading="busy" @click="confirmPasskey">
      {{ t('admin-users-impersonate-passkey') }}
    </AtomButton>

    <!-- Без passkey переключать не на что: остаётся один код. -->
    <button
      v-if="status?.passkey_available"
      type="button"
      class="text-xs text-[var(--noro-muted)] underline underline-offset-4 transition hover:text-[var(--noro-text)]"
      @click="useCode = !useCode"
    >
      {{ useCode ? t('admin-users-impersonate-use-passkey') : t('admin-users-impersonate-use-code') }}
    </button>
  </div>
</template>
