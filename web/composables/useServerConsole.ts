/**
 * Живая консоль игрового сервера.
 *
 * Поток идёт через `fetch` с ручным разбором SSE, а не через `EventSource`:
 * токен админки уходит мастеру заголовком `Authorization`, а `EventSource`
 * заголовки ставить не умеет. Класть токен в query-строку, где он осядет в
 * логах прокси, ради этого не стоит.
 */
export function useServerConsole(gameServerId: string) {
  const api = useApi();
  const base = `/api/admin/game-servers/${gameServerId}/wrapper/console`;

  /** Столько строк держим на экране: дальше DOM начинает подтормаживать. */
  const LIMIT = 2000;

  const lines = ref<string[]>([]);
  const live = ref(false);
  let controller: AbortController | null = null;

  function push(line: string) {
    lines.value.push(line);
    if (lines.value.length > LIMIT) {
      lines.value.splice(0, lines.value.length - LIMIT);
    }
  }

  async function loadBacklog() {
    const data = await api.request<{ lines: string[] }>(base);
    lines.value = data?.lines ?? [];
  }

  async function start() {
    stop();
    controller = new AbortController();
    const response = await fetch(`${api.masterUrl.value}${base}/stream`, {
      headers: { Authorization: `Bearer ${api.token.value}` },
      signal: controller.signal,
    });
    if (!response.ok || !response.body) {
      live.value = false;
      return;
    }
    live.value = true;
    readFrames(response.body.getReader());
  }

  async function readFrames(reader: ReadableStreamDefaultReader<Uint8Array>) {
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        // Событие SSE заканчивается пустой строкой; хвост оставляем в буфере.
        const events = buffer.split("\n\n");
        buffer = events.pop() ?? "";
        for (const event of events) {
          const text = event
            .split("\n")
            .filter((l) => l.startsWith("data:"))
            .map((l) => l.slice(5).replace(/^ /, ""))
            .join("\n");
          if (text) push(text);
        }
      }
    } catch {
      // Поток оборвали — либо ушли со страницы, либо враппер отключился.
    } finally {
      live.value = false;
    }
  }

  function stop() {
    controller?.abort();
    controller = null;
    live.value = false;
  }

  function clear() {
    lines.value = [];
  }

  onBeforeUnmount(stop);

  return { lines, live, loadBacklog, start, stop, clear };
}
