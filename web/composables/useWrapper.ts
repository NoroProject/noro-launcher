import type { PowerAction, WrapperState } from "~/types/wrapper";

/**
 * Состояние враппера и команды ему.
 *
 * Состояние опрашиваем: WebSocket с мастером у админки уже есть, но он про
 * лаунчер, а заводить второй ради двух полей дороже, чем спросить раз в пять
 * секунд. Консоль — отдельный поток, она в {@link useServerConsole}.
 */
export function useWrapper(gameServerId: string) {
  const auth = useAuth();
  const notify = useNotify();
  const base = `/api/admin/game-servers/${gameServerId}/wrapper`;

  const state = ref<WrapperState | null>(null);
  const busy = ref<string | null>(null);
  let poll: ReturnType<typeof setInterval> | null = null;

  async function load() {
    try {
      state.value = await auth.request<WrapperState>(base);
    } catch {
      // Сеть моргнула — прошлое состояние честнее, чем «отключён».
    }
  }

  async function power(action: PowerAction) {
    busy.value = action;
    try {
      await auth.request(`${base}/power`, { method: "POST", body: { action } });
      notify.ok(`Server ${action}`);
    } catch (e) {
      notify.fail(e, `Could not ${action} the server`);
    } finally {
      busy.value = null;
      await load();
    }
  }

  async function command(line: string) {
    if (!line.trim()) return;
    try {
      await auth.request(`${base}/command`, { method: "POST", body: { line } });
    } catch (e) {
      notify.fail(e, "Command rejected");
    }
  }

  const connected = computed(() => state.value?.connected ?? false);
  const running = computed(() => state.value?.status.running ?? false);

  onMounted(() => {
    load();
    poll = setInterval(load, 5000);
  });
  onBeforeUnmount(() => {
    if (poll) clearInterval(poll);
  });

  return { state, busy, connected, running, load, power, command };
}
