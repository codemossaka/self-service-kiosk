import { ref }            from 'vue'
import { invoke }          from '@tauri-apps/api/core'
import { convertFileSrc }  from '@tauri-apps/api/core'
import type { Sermon, PrintResult } from '../types'

export type PrintStatus = 'idle' | 'printing' | 'success' | 'error'

export function usePrinter() {
  const status   = ref<PrintStatus>('idle')
  const errorMsg = ref<string>('')
  const copies   = ref<number>(1)

  /** URL pour l'iframe de prévisualisation */
  function getPdfUrl(source: string, filename: string, sourceType: 'local' | 'remote'): string {
    if (sourceType === 'remote') {
      const base = source.replace(/\/$/, '')
      return `${base}/${encodeURIComponent(filename)}`
    }
    // Chemin local → convertir pour la WebView Tauri
    const sep = source.includes('\\') ? '\\' : '/'
    return convertFileSrc(`${source}${sep}${filename}`)
  }

  function adjustCopies(delta: number): void {
    copies.value = Math.max(1, Math.min(9, copies.value + delta))
  }

  function resetCopies(): void { copies.value = 1 }

  function friendlyPrintError(error: unknown): string {
    const raw = error instanceof Error ? error.message : String(error)

    if (raw.includes('SumatraPDF') || raw.includes('Windows') || raw.includes('PowerShell')) {
      return 'Установите SumatraPDF для печати PDF.'
    }

    return raw.length > 120 ? `${raw.slice(0, 117)}...` : raw
  }

  async function print(
    sermon: Sermon,
    source: string,
    sourceType: 'local' | 'remote'
  ): Promise<void> {
    status.value   = 'printing'
    errorMsg.value = ''

    try {
      let result: PrintResult

      if (sourceType === 'remote') {
        const base = source.replace(/\/$/, '')
        const url  = `${base}/${encodeURIComponent(sermon.filename)}`
        result = await invoke<PrintResult>('print_remote_pdf', { url, copies: copies.value })
      } else {
        result = await invoke<PrintResult>('print_pdf', {
          folder:   source,
          filename: sermon.filename,
          copies:   copies.value,
        })
      }

      if (result.success) {
        status.value = 'success'
        setTimeout(() => { status.value = 'idle' }, 4000)
      } else {
        throw new Error(result.reason ?? 'Ошибка принтера')
      }
    } catch (e: unknown) {
      status.value   = 'error'
      errorMsg.value = friendlyPrintError(e)
      setTimeout(() => { status.value = 'idle' }, 5000)
    }
  }

  return { status, errorMsg, copies, getPdfUrl, adjustCopies, resetCopies, print }
}
