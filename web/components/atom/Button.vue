<script setup lang="ts">
const props = withDefaults(defineProps<{
  variant?: 'primary' | 'secondary' | 'dark' | 'warning' | 'danger' | 'danger-soft' | 'outline' | 'outline-blue' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  icon?: string
  iconRight?: string
  loading?: boolean
  disabled?: boolean
  /** Внутренний роут — рендерится NuxtLink. */
  to?: string
  /** Внешний адрес или скачивание — рендерится обычный `<a>`. */
  href?: string
  /** `true` — имя файла с сервера, строка — своё имя. Только вместе с `href`. */
  download?: boolean | string
  type?: 'button' | 'submit'
  block?: boolean
  /** Одинаковая ширина в ряду однотипных кнопок. */
  equal?: boolean
}>(), {
  variant: 'secondary',
  size: 'md',
  type: 'button',
})

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const slots = useSlots()

/**
 * Кнопка-иконка без подписи. Пустой `<span>` рядом с иконкой съедал бы gap и
 * боковые отступы, из-за чего такие кнопки выходили заметно шире квадрата.
 */
const iconOnly = computed(() => !slots.default)

const isDisabled = computed(() => props.disabled || props.loading)

/**
 * Один тег вместо трёх веток шаблона: копии разметки под button/a/NuxtLink
 * разъезжались бы при каждой правке.
 */
const tag = computed(() => {
  if (props.to) return resolveComponent('NuxtLink')
  if (props.href) return 'a'
  return 'button'
})

const tagAttrs = computed(() => {
  if (props.to) return { to: props.to }
  if (props.href) {
    // `download="true"` браузер принял бы за имя файла — пустая строка значит
    // «оставить имя, которое отдал сервер».
    const name = props.download === true ? '' : props.download
    return { href: props.href, download: name }
  }
  return { type: props.type, disabled: isDisabled.value }
})

const classes = computed(() => {
  const variants: Record<string, string> = {
    primary: 'noro-btn-primary',
    secondary: 'noro-btn-secondary',
    dark: 'noro-btn-dark',
    warning: 'noro-btn-warning',
    danger: 'noro-btn-danger',
    'danger-soft': 'noro-btn-danger-soft',
    outline: 'noro-btn-outline',
    'outline-blue': 'noro-btn-outline-blue',
    ghost: 'noro-btn-dark !bg-transparent hover:!bg-white/5',
  }
  const sizes: Record<string, string> = { sm: 'noro-btn-sm', lg: 'noro-btn-lg' }
  return [
    'noro-btn',
    variants[props.variant] || variants.secondary,
    sizes[props.size] || '',
    {
      'w-full': props.block,
      'noro-btn-equal': props.equal,
      '!px-0 aspect-square': iconOnly.value,
    },
  ]
})

function handleClick(e: MouseEvent) {
  if (isDisabled.value) {
    e.preventDefault()
    return
  }
  emit('click', e)
}
</script>

<template>
  <component
    :is="tag"
    v-bind="tagAttrs"
    :class="classes"
    :aria-disabled="isDisabled || undefined"
    @click="handleClick"
  >
    <UIcon v-if="loading" name="i-lucide-loader-circle" class="size-4 shrink-0 animate-spin" />
    <UIcon v-else-if="icon" :name="icon" class="size-4 shrink-0" />
    <span v-if="!iconOnly" class="truncate"><slot /></span>
    <UIcon v-if="iconRight && !loading" :name="iconRight" class="size-4 shrink-0" />
  </component>
</template>
