<script setup lang="ts">
import type { BuildFileRow } from "~/types/api";

const open = defineModel<boolean>({ required: true });
const props = withDefaults(defineProps<{ unmanaged: string; userManaged: string; files?: BuildFileRow[] | null }>(), {});
const emit = defineEmits<{ apply: [unm: string, usr: string] }>();

type Mode = "always" | "ignored" | "user_override" | "sync_on_change";
const labels: Record<Mode, string> = { always: "Always", ignored: "Ignored", user_override: "User Edit", sync_on_change: "On Change" };
const colors: Record<Mode, string> = {
  always: "bg-[var(--noro-cream)] text-[var(--noro-on-cream)]",
  ignored: "bg-[var(--noro-blue)]/80 text-white",
  user_override: "bg-[var(--noro-magenta)]/80 text-white",
  sync_on_change: "bg-[var(--noro-amber)]/80 text-[var(--noro-bg-deep)]",
};

const search = ref(""); const filter = ref<"all" | Mode>("all");
const customInput = ref("");
const entries = ref<Array<{ path: string; mode: Mode; kind?: string }>>([]);
const hideCore = ref(true);
const CORE_KINDS = ['asset','asset_index','lib','library','client_jar','java','runtime'];
function isCore(e: {kind?:string; path:string}) {
  const k=(e.kind||'').toLowerCase(); if (CORE_KINDS.includes(k)) return true;
  const p=e.path.toLowerCase(); return p.startsWith('assets/')||p.startsWith('libraries/')||p.startsWith('natives/')||p.startsWith('versions/');
}

function init() {
  const map = new Map<string, Mode>();
  props.unmanaged.split("\n").map(s => s.trim()).filter(Boolean).forEach(p => map.set(p, "ignored"));
  props.userManaged.split("\n").map(s => s.trim()).filter(Boolean).forEach(p => { if (!map.has(p)) map.set(p, "user_override"); });
  const known = new Map((props.files || []).map(f => [f.path, f.kind]));
  const res: any[] = [];
  known.forEach((k, p) => res.push({ path: p, mode: map.get(p) || "always", kind: k }));
  map.forEach((m, p) => { if (!known.has(p)) res.push({ path: p, mode: m }); });
  entries.value = res.sort((a, b) => a.path.localeCompare(b.path));
}
watch(open, o => { if (o) init(); });

const list = computed(() => {
  let r = entries.value; if (hideCore.value) r = r.filter(e => !isCore(e));
  if (search.value) r = r.filter(e => e.path.toLowerCase().includes(search.value.toLowerCase()));
  if (filter.value !== "all") r = r.filter(e => e.mode === filter.value);
  return r;
});
const coreHidden = computed(() => hideCore.value ? entries.value.filter(isCore).length : 0);
const counts = computed(() => {
  const c: any = { always: 0, ignored: 0, user_override: 0, sync_on_change: 0 };
  entries.value.forEach(e => c[e.mode]++);
  return c;
});

function set(path: string, m: Mode) { const e = entries.value.find(x => x.path === path); if (e) e.mode = m; }
function addCustom(p?: string) {
  const val = (p ?? customInput.value).trim();
  if (!val) return;
  const ex = entries.value.find(e => e.path === val);
  if (ex) ex.mode = "ignored";
  else entries.value.push({ path: val, mode: "ignored" });
  entries.value.sort((a, b) => a.path.localeCompare(b.path));
  customInput.value = "";
}
function remove(path: string) { entries.value = entries.value.filter(e => e.path !== path); }

function apply() {
  const ign: string[] = [], usr: string[] = [];
  entries.value.forEach(e => { if (e.mode === "ignored") ign.push(e.path); else if (e.mode === "user_override" || e.mode === "sync_on_change") usr.push(e.path); });
  emit("apply", ign.join("\n"), usr.join("\n"));
  open.value = false;
}
function reset() {
  const d = ["saves/","screenshots/","logs/","crash-reports/","options.txt"];
  entries.value.forEach(e => e.mode = d.includes(e.path) ? "ignored" : "always");
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 grid place-items-center bg-[var(--noro-bg-deep)]/95 p-4" @click.self="open=false">
      <div class="noro-panel w-full max-w-3xl overflow-hidden">
        <div class="flex items-center justify-between border-b border-[var(--noro-border)] px-4 py-3">
          <div>
            <div class="noro-pixel text-lg uppercase text-[var(--noro-cream)]">Path Rules</div>
            <div class="text-[10px] uppercase tracking-widest text-[var(--noro-muted)]">File browser • pick sync mode</div>
          </div>
          <UButton icon="i-lucide-x" variant="ghost" @click="open=false" />
        </div>

        <div class="flex items-center gap-2 border-b border-[var(--noro-border)] bg-black/20 px-4 py-2">
          <input v-model="search" class="noro-input-sm w-56" placeholder="Filter paths..." />
          <div class="flex gap-px text-[10px]">
            <button v-for="f in ['all','always','ignored','user_override','sync_on_change'] as const" :key="f"
              class="px-2.5 py-1 font-bold uppercase tracking-widest rounded transition-all"
              :class="filter===f ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]' : 'bg-black/30 text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
              @click="filter = f">{{ f==='all'?'All':labels[f] }}</button>
          </div>

          <button class="ml-1 px-2 py-1 text-[10px] font-bold uppercase tracking-widest rounded transition-all" :class="hideCore ? 'bg-[var(--noro-blue)]/20 text-[var(--noro-blue)]' : 'bg-black/30 text-[var(--noro-muted)] hover:text-[var(--noro-text)]'" @click="hideCore=!hideCore">{{ hideCore ? 'Hide core' : 'Show core' }}</button>

          <div class="ml-auto text-[10px] text-[var(--noro-muted)]">{{ list.length }} shown <span v-if="coreHidden" class="text-[var(--noro-blue)]">({{ coreHidden }} core hidden)</span></div>
        </div>

        <div class="max-h-[48vh] overflow-auto p-2 noro-scroll">
          <div v-for="e in list" :key="e.path"
            class="mb-px flex items-center gap-2 rounded-md border border-[var(--noro-border)] bg-black/10 px-3 py-1.5 text-sm hover:border-[var(--noro-cream)]/50">
            <code class="flex-1 font-mono text-[var(--noro-text)] truncate">{{ e.path }}</code>
            <span v-if="e.kind" class="text-[9px] px-1 text-[var(--noro-muted)]">{{ e.kind }}</span>

            <div class="flex gap-px" @click.stop>
              <button v-for="m in ['always','ignored','user_override','sync_on_change'] as const" :key="m"
                class="px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest rounded transition active:scale-[0.985]"
                :class="e.mode===m ? colors[m] : 'text-[var(--noro-muted)] hover:bg-white/10'"
                @click="set(e.path, m)">{{ labels[m] }}</button>
            </div>
            <button class="text-[var(--noro-muted)] hover:text-[var(--noro-danger)]" @click.stop="remove(e.path)"><UIcon name="i-lucide-x" class="size-3.5"/></button>
          </div>
          <div v-if="!list.length" class="py-6 text-center text-xs text-[var(--noro-muted)]">No matches.</div>
        </div>

        <div class="border-t border-[var(--noro-border)] bg-black/20 p-3">
          <div class="flex gap-2">
            <input v-model="customInput" @keyup.enter="addCustom()" class="noro-input-sm flex-1 font-mono" placeholder="Add custom path (e.g. saves/ )" />
            <AtomButton size="sm" @click="addCustom()">Add</AtomButton>
            <AtomButton size="sm" @click="reset">Defaults</AtomButton>
          </div>
        </div>

        <div class="flex items-center justify-between border-t border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-4 py-3 text-[10px] uppercase text-[var(--noro-muted)]">
          <div class="flex gap-3">
            <span class="text-[var(--noro-cream)]">{{ counts.always }} always</span>
            <span class="text-[var(--noro-blue)]">{{ counts.ignored }} ignored</span>
            <span class="text-[var(--noro-magenta)]">{{ counts.user_override }} user</span>
            <span class="text-[var(--noro-amber)]">{{ counts.sync_on_change }} on-change</span>
          </div>
          <div class="flex gap-2">
            <AtomButton size="sm" @click="open=false">Cancel</AtomButton>
            <AtomButton size="sm" variant="primary" icon="i-lucide-check" @click="apply">
              Apply Rules
            </AtomButton>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.noro-input-sm { background:var(--noro-input); border:0; border-radius:6px; color:var(--noro-text); font-size:.8125rem; padding:.3rem .6rem; }
.noro-scroll { scrollbar-width: thin; }
</style>
