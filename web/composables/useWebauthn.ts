/**
 * WebAuthn: перевод между тем, что отдаёт браузер (ArrayBuffer), и тем, что
 * ждёт сервер (base64url).
 *
 * Ручной перевод, а не библиотека: обеих сторон здесь по тридцать строк, а
 * формат задан спецификацией и не меняется.
 */

function b64urlToBuf(value: string): Uint8Array {
  const padded = value.replace(/-/g, '+').replace(/_/g, '/')
  const raw = atob(padded + '='.repeat((4 - (padded.length % 4)) % 4))
  return Uint8Array.from(raw, (c) => c.charCodeAt(0))
}

function bufToB64url(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf)
  let binary = ''
  // Порциями: спред на большом массиве упирается в лимит аргументов функции.
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000))
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
}

/** Ответ сервера на `/options`: опции для браузера плюс id придержанного состояния. */
export interface ChallengeRes {
  state_id: string
  publicKey: any
}

/** Создать ключ. Возвращает то, что уходит в `/register/verify`. */
export async function createCredential(opts: ChallengeRes) {
  const publicKey = { ...opts.publicKey }
  publicKey.challenge = b64urlToBuf(publicKey.challenge)
  publicKey.user = { ...publicKey.user, id: b64urlToBuf(publicKey.user.id) }
  publicKey.excludeCredentials = (publicKey.excludeCredentials || []).map((c: any) => ({
    ...c,
    id: b64urlToBuf(c.id),
  }))

  const cred = (await navigator.credentials.create({ publicKey })) as PublicKeyCredential
  if (!cred) throw new Error('Authenticator returned nothing')
  const response = cred.response as AuthenticatorAttestationResponse

  return {
    id: cred.id,
    rawId: bufToB64url(cred.rawId),
    type: cred.type,
    extensions: {},
    response: {
      attestationObject: bufToB64url(response.attestationObject),
      clientDataJSON: bufToB64url(response.clientDataJSON),
    },
  }
}

/** Подписать вызов существующим ключом. Уходит в `/login/verify`. */
export async function getCredential(opts: ChallengeRes) {
  const publicKey = { ...opts.publicKey }
  publicKey.challenge = b64urlToBuf(publicKey.challenge)
  publicKey.allowCredentials = (publicKey.allowCredentials || []).map((c: any) => ({
    ...c,
    id: b64urlToBuf(c.id),
  }))

  const cred = (await navigator.credentials.get({ publicKey })) as PublicKeyCredential
  if (!cred) throw new Error('Authenticator returned nothing')
  const response = cred.response as AuthenticatorAssertionResponse

  return {
    id: cred.id,
    rawId: bufToB64url(cred.rawId),
    type: cred.type,
    extensions: {},
    response: {
      authenticatorData: bufToB64url(response.authenticatorData),
      clientDataJSON: bufToB64url(response.clientDataJSON),
      signature: bufToB64url(response.signature),
      userHandle: response.userHandle ? bufToB64url(response.userHandle) : null,
    },
  }
}
