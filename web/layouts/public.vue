<script setup lang="ts">
/** Каркас страниц, открытых без входа: шапка, содержимое, подвал. */
</script>

<template>
  <!-- Без своей заливки: цвет даёт `body`, а сплошной фон здесь перекрыл бы
       фоновый слой приложения. -->
  <div class="flex min-h-screen flex-col text-[var(--noro-text)]">
    <PublicNavbar />
    <main class="flex-1">
      <slot />
    </main>
    <PublicFooter />
  </div>
</template>
