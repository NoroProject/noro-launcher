<script setup lang="ts">
defineProps<{
    title: string;
    subtitle?: string;
}>();
</script>

<template>
    <div class="min-h-screen lg:h-screen lg:overflow-hidden lg:flex bg-[var(--noro-bg-deep)]">
        <NoroSidebar />

        <main class="flex min-w-0 flex-1 flex-col lg:h-screen lg:overflow-hidden">
            <!-- z-30, а не z-10: контент часто использует z-10 для локальной
                 раскладки, и при равном значении он выигрывал по порядку в DOM
                 и перекрывал шапку. -->
            <header class="shrink-0 z-30 bg-[var(--noro-bg-deep)] px-5 md:px-8 border-b border-[var(--noro-border)]">
                <div class="mx-auto flex min-h-20 max-w-[1440px] items-center justify-between gap-5">
                    <div class="min-w-0">
                        <h1 class="noro-pixel truncate text-2xl uppercase text-[var(--noro-cream)]">
                        <slot name="title">{{ $props.title }}</slot>
                        </h1>
                        <p
                            v-if="$props.subtitle"
                            class="mt-1 truncate text-sm font-semibold uppercase tracking-wider text-[var(--noro-muted)]"
                        >
                            {{ $props.subtitle }}
                        </p>
                    </div>
                    <div class="noro-header-actions flex items-center gap-2">
                        <slot name="actions" />
                    </div>
                </div>
            </header>

            <section class="flex-1 min-h-0 overflow-y-auto lg:overflow-hidden flex flex-col mx-auto max-w-[1440px] w-full p-5 md:p-8">
                <slot />
            </section>
        </main>
    </div>
</template>

<style scoped>
.noro-header-actions :deep(a),
.noro-header-actions :deep(button) {
  align-items: center;
  border-radius: 8px;
  display: inline-flex;
  min-height: 44px;
  padding: 0 14px;
  font-weight: 700;
  text-transform: uppercase;
  font-size: 0.8rem;
  letter-spacing: 0.02em;
}
.noro-header-actions :deep(.noro-btn) {
  min-height: 44px;
}
</style>
