export default defineAppConfig({
  ui: {
    colors: {
      primary: 'neutral',
      neutral: 'slate',
      success: 'green',
      warning: 'amber',
      error: 'red'
    },
    /**
     * Тост в нашей палитре, а не в дефолтной для Nuxt UI.
     *
     * Раньше он приезжал светлой карточкой с толстой зелёной полосой во всю
     * ширину и пустотой в половину высоты: одна короткая строка занимала место
     * абзаца, а три подряд закрывали угол экрана. Теперь это узкая плашка в
     * одну строку, статус читается цветной полосой слева и значком, а всё
     * остальное — обычный текст сайта.
     *
     * Цвета берутся из токенов темы: `success`/`error` из палитры Nuxt UI
     * рядом с нашим фоном выглядели как чужой элемент.
     */
    toast: {
      slots: {
        root: [
          'relative group overflow-hidden w-full',
          'bg-[var(--noro-panel)] ring-0 border border-[var(--noro-border)]',
          'rounded-[var(--noro-r-md)] shadow-lg',
          // Полоса статуса слева — 4px по сетке; цвет задаёт вариант ниже.
          'before:absolute before:inset-y-0 before:left-0 before:w-1',
          'p-3 pl-4 flex items-center gap-3 focus:outline-none'
        ].join(' '),
        wrapper: 'w-0 flex-1 flex flex-col gap-0.5',
        title: 'text-xs font-bold leading-4 text-[var(--noro-text)]',
        description: 'text-[11px] leading-4 text-[var(--noro-muted)]',
        icon: 'shrink-0 size-4',
        actions: 'flex gap-1.5 shrink-0 items-center',
        progress: 'absolute inset-x-0 bottom-0',
        close: 'p-0 size-4 text-[var(--noro-muted)] hover:text-[var(--noro-text)]'
      },
      variants: {
        color: {
          success: {
            root: 'before:bg-[var(--noro-green)]',
            icon: 'text-[var(--noro-green)]'
          },
          error: {
            root: 'before:bg-[var(--noro-danger)]',
            icon: 'text-[var(--noro-danger)]'
          },
          warning: {
            root: 'before:bg-[var(--noro-amber)]',
            icon: 'text-[var(--noro-amber)]'
          },
          info: {
            root: 'before:bg-[var(--noro-blue)]',
            icon: 'text-[var(--noro-blue)]'
          },
          neutral: {
            root: 'before:bg-[var(--noro-border)]',
            icon: 'text-[var(--noro-muted)]'
          }
        }
      }
    }
  }
})
