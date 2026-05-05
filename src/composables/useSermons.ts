import { ref, computed } from 'vue'
import { invoke }        from '@tauri-apps/api/core'
import { open }          from '@tauri-apps/plugin-dialog'
import type { Sermon, Config } from '../types'

// ─── module-level state (singleton) ───────────────────
const sermons    = ref<Sermon[]>([])
const source     = ref<string>('')            // chemin local ou URL https://
const sourceType = ref<'local' | 'remote'>('local')

// ─── Parsing nom de fichier (miroir du Rust) ──────────
const MONTHS_RU = ['янв.','фев.','мар.','апр.','мая','июн.',
                   'июл.','авг.','сен.','окт.','ноя.','дек.']

function parseFilename(filename: string): Sermon | null {
  const base = filename.replace(/\.pdf$/i, '')
  const m = base.match(/^(\d{2}-\d{4}[A-Za-z]?)\s*[-–_]?\s*(.+)$/i)
  if (!m) return null

  const code  = m[1].toUpperCase()
  const title = m[2].replace(/_/g, ' ').trim()
  const yy    = parseInt(code.substring(0, 2))
  const year  = yy >= 40 ? 1900 + yy : 2000 + yy
  const mm    = parseInt(code.substring(3, 5))
  const dd    = parseInt(code.substring(5, 7))
  const date  = `${dd} ${MONTHS_RU[mm - 1] ?? '?'} ${year}`

  return { code, title, date, year, filename, lieu: 'Jeffersonville, IN' }
}

export function useSermons() {

  // ─── Config ─────────────────────────────────────────
  async function loadConfig(): Promise<Config> {
    return invoke<Config>('get_config')
  }
  async function saveConfig(cfg: Config): Promise<void> {
    await invoke('save_config', { config: cfg })
  }

  // ─── Source locale (dossier) ────────────────────────
  async function selectLocalFolder(): Promise<string | null> {
    const selected = await open({ title: 'Выберите папку с PDF-проповедями', directory: true, multiple: false })
    if (!selected || typeof selected !== 'string') return null
    await loadLocalFolder(selected)
    await saveConfig({ source: selected, sourceType: 'local' })
    return selected
  }

  async function loadLocalFolder(path: string): Promise<void> {
    const list = await invoke<Sermon[]>('read_sermons', { folder: path })
    sermons.value    = list
    source.value     = path
    sourceType.value = 'local'
  }

  // ─── Source distante (https://) ─────────────────────
  async function loadRemoteUrl(url: string): Promise<void> {
    const base = url.replace(/\/$/, '')
    let list: Sermon[] = []

    // 1. Essayer sermons.json
    try {
      const r = await fetch(`${base}/sermons.json`)
      if (r.ok) {
        list = await r.json() as Sermon[]
      }
    } catch { /* pas de manifest */ }

    // 2. Fallback : parser le listing HTML du répertoire
    if (!list.length) {
      const html = await fetch(`${base}/`).then(r => r.text())
      const doc  = new DOMParser().parseFromString(html, 'text/html')
      const hrefs = [...doc.querySelectorAll('a[href]')]
        .map(a => decodeURIComponent(a.getAttribute('href') ?? ''))
        .filter(h => /\.pdf$/i.test(h) && !h.startsWith('?') && !h.startsWith('/'))
        .map(h => h.replace(/^.*\//, ''))   // nom du fichier seulement

      list = hrefs.map(parseFilename).filter(Boolean) as Sermon[]
    }

    if (!list.length) throw new Error('Ни одного PDF не найдено по адресу: ' + url)

    sermons.value    = list.sort((a, b) => a.code.localeCompare(b.code))
    source.value     = base
    sourceType.value = 'remote'
  }

  async function setRemoteUrl(url: string): Promise<void> {
    await loadRemoteUrl(url)
    await saveConfig({ source: url, sourceType: 'remote' })
  }

  // ─── Init au démarrage ──────────────────────────────
  async function init(): Promise<boolean> {
    try {
      const cfg = await loadConfig()
      if (!cfg.source) return false

      if (cfg.sourceType === 'remote') {
        await loadRemoteUrl(cfg.source)
      } else {
        await loadLocalFolder(cfg.source)
      }
      return true
    } catch {
      return false
    }
  }

  // ─── Recherche & filtres ─────────────────────────────
  function search(query: string): Sermon[] {
    const q = query.toLowerCase().trim()
    if (!q) return []
    return sermons.value.filter(
      s => s.title.toLowerCase().includes(q) || s.code.toLowerCase().includes(q)
    )
  }

  const years = computed(() => {
    const set = new Set(sermons.value.map(s => s.year))
    return [...set].sort((a, b) => b - a)
  })

  function byYear(year: number): Sermon[] {
    return sermons.value.filter(s => s.year === year)
  }

  const totalCount = computed(() => sermons.value.length)
  const yearRange  = computed(() => {
    if (!years.value.length) return ''
    return `${Math.min(...years.value)} – ${Math.max(...years.value)}`
  })

  return {
    sermons,
    source,
    sourceType,
    years,
    totalCount,
    yearRange,
    init,
    selectLocalFolder,
    setRemoteUrl,
    search,
    byYear,
  }
}