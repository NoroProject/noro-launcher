/**
 * Позиция всплывающего списка, привязанного к полю.
 *
 * Список рисуется через `Teleport` в `body` и позиционируется в координатах
 * окна: внутри панели его обрезал бы любой предок с `overflow`, а такие
 * предки в админке встречаются почти на каждой странице.
 */
export function useAnchoredPopup(
  anchor: Ref<HTMLElement | null>,
  open: Ref<boolean>
) {
  const style = ref<Record<string, string>>({})

  function update() {
    const el = anchor.value
    if (!el || !open.value) return
    const rect = el.getBoundingClientRect()
    const below = window.innerHeight - rect.bottom
    // Вверх — только если снизу заведомо тесно, а сверху просторнее.
    const up = below < 280 && rect.top > below
    style.value = {
      position: 'fixed',
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      maxHeight: `${Math.max(160, (up ? rect.top : below) - 16)}px`,
      ...(up
        ? { bottom: `${window.innerHeight - rect.top + 8}px` }
        : { top: `${rect.bottom + 8}px` }),
    }
  }

  watch(open, () => update(), { flush: 'post' })

  onMounted(() => {
    // `true` — ловим прокрутку любого контейнера, не только окна.
    window.addEventListener('scroll', update, true)
    window.addEventListener('resize', update)
  })
  onBeforeUnmount(() => {
    window.removeEventListener('scroll', update, true)
    window.removeEventListener('resize', update)
  })

  return { style, update }
}
