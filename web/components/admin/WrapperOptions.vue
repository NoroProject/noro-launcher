<script setup lang="ts">
interface Option {
  key: string
  fallback: string
  required?: boolean
  note: string
}

/** Полный набор ключей noro-wrapper.properties — то же, что читает WrapperConfig. */
const OPTIONS: Option[] = [
  {
    key: 'master-url',
    fallback: '—',
    required: true,
    note: 'Master node address. A trailing slash is stripped.',
  },
  {
    key: 'secret',
    fallback: '—',
    required: true,
    note: 'Agent secret for this game server, issued in the build admin panel under Game servers. Must start with noroagent_. Lives only here — the wrapper hands it to the server process through an environment variable.',
  },
  {
    key: 'server-jar',
    fallback: '—',
    required: true,
    note: 'Server jar, relative to server-dir. Starts with @ for an args file instead — NeoForge and Forge launch that way: @libraries/net/neoforged/neoforge/21.1.248/unix_args.txt',
  },
  {
    key: 'signing-public-key',
    fallback: 'fetch and pin',
    note: 'Master ed25519 key, hex. Empty means fetch once and pin to noro/signing-key.pub; a later change becomes a hard error. Set it explicitly in production — a pinned key here is the real trust anchor.',
  },
  {
    key: 'server-dir',
    fallback: '.',
    note: 'Server directory. Everything else resolves against it, and the agent goes into its plugins/ or mods/.',
  },
  {
    key: 'java',
    fallback: 'java',
    note: 'Java binary. Point it at a specific JDK when the default one is the wrong version for this Minecraft release.',
  },
  {
    key: 'jvm-args',
    fallback: '-Xmx4G',
    note: 'JVM arguments, split on whitespace. Accepts an @-file: NeoForge keeps its own as @user_jvm_args.txt.',
  },
  {
    key: 'server-args',
    fallback: 'nogui',
    note: 'Arguments passed after the jar or args file. Set empty to pass none.',
  },
  {
    key: 'platform',
    fallback: 'detected',
    note: 'paper, fabric, neoforge or forge. Overrides detection — needed when several loader versions sit in libraries/ and the guess is ambiguous.',
  },
  {
    key: 'mc-version',
    fallback: 'detected',
    note: 'Minecraft version, e.g. 1.21.1. Overrides detection.',
  },
]
</script>

<template>
  <section class="noro-panel p-5">
    <div class="mb-4 flex items-center gap-3">
      <div class="grid size-10 place-items-center rounded bg-[var(--noro-amber)]/10 text-[var(--noro-amber)]">
        <UIcon name="i-lucide-sliders-horizontal" class="size-5" />
      </div>
      <div>
        <h3 class="text-lg font-black text-white">noro-wrapper.properties</h3>
        <p class="text-sm text-[var(--noro-muted)]">Every option the wrapper reads.</p>
      </div>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr class="border-b border-[var(--noro-border)] text-left text-xs uppercase text-[var(--noro-muted)]">
            <th class="py-2 pr-4 font-bold">Key</th>
            <th class="py-2 pr-4 font-bold">Default</th>
            <th class="py-2 font-bold">Meaning</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="option in OPTIONS"
            :key="option.key"
            class="border-b border-[var(--noro-border-soft)] align-top last:border-0"
          >
            <td class="py-3 pr-4 font-mono text-xs whitespace-nowrap text-[var(--noro-cream)]">
              {{ option.key }}
            </td>
            <td class="py-3 pr-4 text-xs whitespace-nowrap">
              <span v-if="option.required" class="font-bold text-[var(--noro-amber)]">required</span>
              <span v-else class="font-mono text-[var(--noro-muted)]">{{ option.fallback }}</span>
            </td>
            <td class="py-3 text-[var(--noro-muted)]">{{ option.note }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
