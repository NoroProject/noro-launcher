<script setup lang="ts">
/** Каркас страниц, открытых без входа: шапка, содержимое, подвал. */
</script>

<template>
  <div class="flex min-h-screen flex-col bg-[var(--noro-bg)] text-[var(--noro-text)]">
    <PublicNavbar />
    <main class="flex-1">
      <slot />
    </main>
    <PublicFooter />
  </div>
</template>
