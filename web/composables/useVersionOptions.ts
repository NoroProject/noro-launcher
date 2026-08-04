export function useVersionOptions() {
  const api = useApi()
  const minecraft = useState<string[]>('noro_mc_versions', () => [
    '1.21.8', '1.21.7', '1.21.6', '1.21.5', '1.21.4', '1.21.3',
    '1.21.1', '1.20.6', '1.20.4', '1.20.1', '1.19.4', '1.18.2', '1.16.5'
  ])
  const loading = useState<boolean>('noro_versions_loading', () => false)
  const loader = useState<Record<string, string[]>>('noro_loader_versions', () => ({
    vanilla: [''],
    fabric: ['0.16.14', '0.16.10', '0.15.11'],
    neoforge: ['21.1.233', '21.1.209', '21.1.172', '21.4.137'],
    forge: ['52.0.28', '51.0.33', '47.4.0', '40.3.0'],
    quilt: ['0.29.1', '0.28.0']
  }))

  async function loadMinecraft() {
    loading.value = true
    try {
      const data = await api.request<{ versions: string[] }>('/api/admin/versions/minecraft')
      if (data.versions?.length) minecraft.value = data.versions
    } finally {
      loading.value = false
    }
  }

  async function loadLoader(kind: string, mc: string) {
    if (!kind || kind === 'vanilla' || !mc) return
    const data = await api.request<{ versions: string[] }>(
      `/api/admin/versions/loader/${kind}?mc=${encodeURIComponent(mc)}`
    )
    if (data.versions?.length) loader.value = { ...loader.value, [kind]: data.versions }
  }

  return { minecraft, loader, loading, loadMinecraft, loadLoader }
}
