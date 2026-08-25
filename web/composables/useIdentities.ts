/**
 * Привязки аккаунта к платформам: как их показывать и как ими управлять.
 *
 * Раньше вместо этого было три поля `discord_*` прямо в профиле, и каждая
 * страница доставала аватар по-своему. Теперь платформ несколько, и место,
 * где решается «чей аватар и какой ник показать», должно быть одно.
 */

import type { UserIdentity, UserProfile } from '~/types/api'

export interface ProviderMeta {
  label: string
  icon: string
  /**
   * Цвет логотипа. Не наше оформительское решение: у Discord и Twitch
   * гайдлайны разрешают марку только в фирменном цвете, белом или чёрном.
   */
  color: string
  /**
   * Логотип одноцветный и красится через `currentColor`. У Google — нет:
   * его «G» существует только полноцветной или в официальных белом и
   * нейтральном вариантах, перекрашивать её в один цвет запрещено.
   */
  mono: boolean
}

/** Логотипы — своя коллекция `brand`, см. `nuxt.config.ts`. */
export const PROVIDER_META: Record<string, ProviderMeta> = {
  discord: { label: 'Discord', icon: 'i-brand-discord', color: '#5865F2', mono: true },
  twitch: { label: 'Twitch', icon: 'i-brand-twitch', color: '#9146FF', mono: true },
  google: { label: 'Google', icon: 'i-brand-google', color: '', mono: false },
  telegram: { label: 'Telegram', icon: 'i-brand-telegram', color: '#24A1DE', mono: true },
}

export function providerMeta(provider: string): ProviderMeta {
  return (
    PROVIDER_META[provider] ?? {
      label: provider,
      icon: 'i-lucide-circle-user',
      color: 'var(--noro-muted)',
      mono: true,
    }
  )
}

/** Стиль для тега логотипа: полноцветную марку не трогаем. */
export function providerIconStyle(provider: string) {
  const meta = providerMeta(provider)
  return meta.mono ? { color: meta.color } : {}
}

type MaybeUser = Pick<UserProfile, 'identities' | 'username'> | null | undefined

/** Платформа регистрации: с неё берутся ник и аватар по умолчанию. */
export function primaryIdentity(user: MaybeUser): UserIdentity | null {
  const list = user?.identities ?? []
  return list.find((i) => i.is_primary) ?? list[0] ?? null
}

/** Ник вне игры. `null` у локального аккаунта — платформы у него нет. */
export function identityHandle(user: MaybeUser): string | null {
  return primaryIdentity(user)?.username ?? null
}

/** Аватар: с первичной платформы, а если там пусто — с любой другой. */
export function identityAvatar(user: MaybeUser): string | null {
  return (
    primaryIdentity(user)?.avatar_url ??
    (user?.identities ?? []).find((i) => i.avatar_url)?.avatar_url ??
    null
  )
}

export function useIdentities() {
  const auth = useAuth()

  const identities = computed(() => auth.user.value?.identities ?? [])

  /** Уйти на платформу за подтверждением привязки. */
  async function startLink(provider: string) {
    const res = await auth.request<{ url: string }>('/api/me/identities/link', {
      method: 'POST',
      body: { provider, redirect: `${window.location.origin}/cabinet/settings` },
    })
    window.location.href = res.url
  }

  async function unlink(provider: string) {
    await auth.request(`/api/me/identities/${provider}`, { method: 'DELETE' })
    await auth.loadMe()
  }

  return { identities, startLink, unlink, providerMeta }
}
