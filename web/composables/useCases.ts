/**
 * Очередь дел и карточка одного разбора.
 *
 * Страница живёт пушем, а не опросом: мастер шлёт `CaseUpdated`, когда с делом
 * что-то произошло, и карточка перечитывается ровно тогда. Опрос раз в пять
 * секунд и грузил мастер впустую, и всё равно отставал настолько, что модератор
 * успевал нажать кнопку дважды.
 */

import type { CaseDetail, CaseRow } from '~/types/cases'

/**
 * Очередь: поиск и страницы держит `usePagedList`, то есть считает их мастер.
 *
 * Раньше сюда приезжала вся очередь, и фильтр применялся к загруженному
 * массиву — дело за пределами выдачи не находилось, и это выглядело как «такого
 * дела нет». Тем же переездом лечились остальные списки админки.
 */
export function useCaseQueue() {
  const socket = useAdminSocket()
  let unsubscribe: (() => void) | null = null

  const openOnly = ref(true)
  const paged = usePagedList<CaseRow>('admin-cases', '/api/admin/cases', {
    perPage: 50,
    params: () => ({ open_only: String(openOnly.value) }),
  })

  // Смена галочки — это другой запрос, и начинать его надо с первой страницы.
  watch(openOnly, () => paged.goTo(1))

  onMounted(() => {
    // Очередь меняется от тех же событий, что и карточки: взяли дело, закрыли,
    // завели новое. Кадр приходит на любое дело — перечитываем ту страницу, на
    // которой модератор стоит, а не выбрасываем его на первую.
    unsubscribe = socket.subscribe((frame) => {
      if (frame.t === 'CaseUpdated') paged.refresh()
    })
  })
  onUnmounted(() => unsubscribe?.())

  return { ...paged, cases: paged.items, openOnly }
}

export function useCase(id: MaybeRefOrGetter<string>) {
  const auth = useAuth()
  const notify = useNotify()

  const socket = useAdminSocket()
  let unsubscribe: (() => void) | null = null

  const detail = ref<CaseDetail | null>(null)
  const pending = ref(false)

  async function load(quiet = false) {
    if (!quiet) pending.value = true
    try {
      detail.value = await auth.request<CaseDetail>(`/api/admin/cases/${toValue(id)}`)
    } catch (e) {
      // Перечитывание по пушу молчит: сеть моргнула — карточка останется
      // прежней, а всплывающая ошибка на каждое событие дела сделает страницу
      // неюзабельной ровно в тот момент, когда по ней работают.
      if (!quiet) notify.fail(e, 'Failed to load case')
    } finally {
      pending.value = false
    }
  }

  /**
   * Действие над делом.
   *
   * Неуспех приезжает статусом, а не полем в теле: «дело уже взято» — это 409, и
   * `$fetch` бросает сам. Раньше мастер отвечал 200 с `{"ok": false}`, и забыть
   * проверить это поле значило показать «готово» там, где ничего не произошло.
   */
  async function act(path: string, method: 'POST' | 'PUT' = 'POST', body?: Record<string, unknown>) {
    try {
      await auth.request(`/api/admin/cases/${toValue(id)}/${path}`, { method, body })
      notify.ok()
      await load(true)
      return true
    } catch (e) {
      notify.fail(e)
      // Дело разошлось с тем, что на экране, — показываем свежее состояние.
      if (apiErrorCode(e) === 'already_claimed' || apiErrorCode(e) === 'case_not_open') {
        await load(true)
      }
      return false
    }
  }

  onMounted(() => {
    load()
    unsubscribe = socket.subscribe((frame) => {
      // Кадр приходит на любое дело: мастер не знает, какое открыто в этой
      // вкладке. Чужое пропускаем — за карточкой ходить незачем.
      if (frame.t === 'CaseUpdated' && frame.d.case_id === toValue(id)) load(true)
    })
  })
  onUnmounted(() => unsubscribe?.())

  return { detail, pending, load, act }
}
