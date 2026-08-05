<script setup lang="ts">
import type { AgentFile } from '~/types/agent'

const props = defineProps<{ wrapper: AgentFile | null; host: string }>()

const masterUrl = computed(() => `https://${props.host}`)
</script>

<template>
  <section class="noro-panel p-5">
    <div class="mb-4 flex items-center gap-3">
      <div class="grid size-10 place-items-center rounded bg-[var(--noro-blue)]/10 text-[var(--noro-blue)]">
        <UIcon name="i-lucide-terminal" class="size-5" />
      </div>
      <h3 class="text-lg font-black text-white">Set up a game server</h3>
    </div>

    <div class="grid gap-5 text-[var(--noro-muted)]">
      <p>
        ServerWrapper installs the right agent for your server, wires up
        <code>authlib-injector</code>, and keeps the server visible in the launcher while it boots.
        It is an installer and a supervisor — access control itself stays on the master.
      </p>

      <div class="grid gap-5">
        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">1. Download the wrapper</h4>
          <p class="mb-3 text-sm">Put it next to your server jar.</p>
          <AtomButton
            v-if="wrapper"
            variant="primary"
            icon="i-lucide-download"
            :href="wrapper.url"
            download="wrapper.jar"
            class="inline-flex w-auto"
          >
            Download wrapper.jar
          </AtomButton>
          <p v-else class="text-sm text-[var(--noro-amber)]">
            Not built yet — run <code>./gradlew collectAgents</code> in <code>agent/</code> and copy
            <code>agent/build/agents/</code> into <code>{NORO_DATA_DIR}/agents/</code>.
          </p>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">2. Create noro-wrapper.properties</h4>
          <p class="mb-2 text-sm">
            The agent secret is issued per game server in the build admin panel, section
            <strong>Game servers</strong>. It lives here and nowhere else — the wrapper passes it to
            the server process itself.
          </p>
          <pre class="overflow-x-auto rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs text-[var(--noro-cream)]">master-url={{ masterUrl }}
secret=noroagent_...
server-jar=paper-1.21.1.jar
jvm-args=-Xmx4G -Xms4G</pre>
          <p class="mt-2 text-sm">
            That is the minimum. NeoForge and Forge start from an args file, not a jar — pass it
            with a leading <code>@</code>, for example
            <code>server-jar=@libraries/net/neoforged/neoforge/21.1.248/unix_args.txt</code>, and
            keep their own JVM file as <code>jvm-args=@user_jvm_args.txt</code>. Every other option
            is listed below.
          </p>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">3. Start the server through it</h4>
          <p class="mb-2 text-sm">
            The wrapper detects the platform and Minecraft version, installs the matching agent,
            verifies its signature, and launches the server.
          </p>
          <div class="rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs break-all text-[var(--noro-cream)]">
            java -jar wrapper.jar
          </div>
        </div>

        <div>
          <h4 class="mb-2 font-bold text-[var(--noro-text)]">4. Keep online-mode on</h4>
          <p class="text-sm">
            Sessions are validated against this master, so the server must stay in online mode.
          </p>
          <div class="mt-2 rounded border border-[var(--noro-border)] bg-black/30 p-3 font-mono text-xs text-[var(--noro-cream)]">
            online-mode=true
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
