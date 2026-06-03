<template>
  <div class="app">

    <!-- ═══════ HEADER ═══════ -->
    <header class="hdr">
      <div class="brand">
        <span class="brand-cross">✝</span>
        <div>
          <div class="brand-name">Библиотека</div>
          <div class="brand-sub">Библиотека данных</div>
        </div>
      </div>

      <div class="hdr-center">
        <div v-if="screen !== 's-home' && screen !== 's-setup'" class="hdr-page-title">
          {{ headerTitle }}
        </div>
        <div v-if="headerSub" class="hdr-page-sub">{{ headerSub }}</div>
      </div>

      <div class="hdr-right">
        <template v-if="screen === 's-home' || screen === 's-setup'">
          <span class="hdr-clock">{{ clock }}</span>
          <button v-if="screen === 's-home'" class="folder-btn" @click="screen = 's-setup'">
            ⚙ Источник
          </button>
        </template>
        <button v-else class="back-btn" @click="goBack">← {{ backLabel }}</button>
      </div>
    </header>

    <!-- ═══════ SCREENS ═══════ -->
    <main class="screens">

      <!-- ── НАСТРОЙКА ИСТОЧНИКА ── -->
      <section v-if="screen === 's-setup'" class="scr scr-center">
        <div class="setup-box">
          <div class="setup-cross">✝</div>
          <h1 class="setup-title">Источник данных</h1>
          <p class="setup-sub">Выберите локальную папку или укажите адрес удалённого сервера</p>

          <!-- Вкладки -->
          <div class="setup-tabs">
            <button
              :class="['stab', { active: setupTab === 'local' }]"
              @click="setupTab = 'local'"
            >📁 Локальная папка</button>
            <button
              :class="['stab', { active: setupTab === 'remote' }]"
              @click="setupTab = 'remote'"
            >🌐 Удалённый сервер</button>
          </div>

          <!-- Локальная папка -->
          <div v-if="setupTab === 'local'" class="setup-section">
            <p class="setup-hint">
              Формат имён файлов : <strong>63-0318 The First Seal.pdf</strong>
            </p>
            <div v-if="currentSource && sourceType === 'local'" class="setup-current">
              <span class="setup-current-icon">📁</span>
              <span class="setup-current-path">{{ currentSource }}</span>
            </div>
            <button class="setup-btn" :disabled="loadingSource" @click="handleSelectFolder">
              <span>{{ loadingSource ? '⏳ Загрузка…' : '📂 Выбрать папку' }}</span>
            </button>
          </div>

          <!-- Удалённый сервер -->
          <div v-else class="setup-section">
            <p class="setup-hint">
              URL папки с PDF-файлами.<br>
              Поддерживается листинг директории или файл <code>sermons.json</code>.
            </p>
            <div v-if="currentSource && sourceType === 'remote'" class="setup-current">
              <span class="setup-current-icon">🌐</span>
              <span class="setup-current-path">{{ currentSource }}</span>
            </div>
            <div class="url-input-wrap">
              <input
                v-model="remoteUrl"
                class="url-input"
                type="url"
                placeholder="https://example.com/sermons/"
                @keydown.enter="handleSetRemote"
              />
            </div>
            <button class="setup-btn" :disabled="loadingSource || !remoteUrl.trim()" @click="handleSetRemote">
              <span>{{ loadingSource ? '⏳ Подключение…' : '🌐 Подключиться' }}</span>
            </button>
          </div>

          <p v-if="setupError" class="setup-error">{{ setupError }}</p>
        </div>
      </section>

      <!-- ── ГЛАВНАЯ ── -->
      <section v-else-if="screen === 's-home'" class="scr">
        <div class="home-body">
          <p class="home-title">Найдите <span>что вам нужно</span> для печати</p>

          <div class="search-wrap" :class="{ focused: searchFocused }">
            <span class="sw-icon">🔍</span>
            <input
              ref="searchInputRef"
              v-model="query"
              class="sw-input"
              type="text"
              placeholder="Введите название…"
              autocomplete="off"
              spellcheck="false"
              @focus="searchFocused = true"
              @blur="searchFocused = false"
              @keydown.enter="doSearch"
              @keydown.escape="query = ''"
            />
            <span v-if="query" class="sw-clear" @mousedown.prevent="query = ''">✕</span>
          </div>

          <!-- Клавиатура ЙЦУКЕН -->
          <div class="kbd">
            <div v-for="(row, ri) in KBD_ROWS" :key="ri" class="kbd-row">
              <button
                v-for="key in row"
                :key="key"
                class="key"
                @mousedown.prevent="kbdPress(key)"
              >{{ key }}</button>
            </div>
            <div class="kbd-row">
              <button class="key key-fn"  @mousedown.prevent="kbdPress('⌫')">⌫ Стереть</button>
              <button class="key key-spc" @mousedown.prevent="kbdPress(' ')">ПРОБЕЛ</button>
              <button class="key key-go"  @mousedown.prevent="doSearch">Искать ›</button>
            </div>
          </div>

          <button class="yr-browse" @click="navigate('s-years')">
            <span class="yrb-icon">📅</span>
            <span class="yrb-body">
              <span class="yrb-title">Просмотр по годам</span>
              <span class="yrb-sub">{{ yearRangeLabel }}</span>
            </span>
            <span class="yrb-arr">›</span>
          </button>
        </div>
      </section>

      <!-- ── РЕЗУЛЬТАТЫ ПОИСКА ── -->
      <section v-else-if="screen === 's-results'" class="scr">
        <div class="qbar">
          <div class="qb-box" :class="{ 'qb-box--editing': editingQuery }">
            <span class="qb-icon">🔍</span>
            <template v-if="editingQuery">
              <input
                ref="resultsInputRef"
                v-model="query"
                class="qb-input"
                type="text"
                autocomplete="off"
                spellcheck="false"
                @keydown.enter="doSearchInline"
                @keydown.escape="cancelEditQuery"
              />
              <span v-if="query" class="sw-clear qb-clear" @mousedown.prevent="query = ''">✕</span>
            </template>
            <span v-else class="qb-text">{{ lastQuery }}</span>
            <button v-if="!editingQuery" class="qb-mod" @click="startEditQuery">✎ Изменить</button>
            <button v-else class="qb-mod qb-mod--cancel" @mousedown.prevent="cancelEditQuery">✕ Отмена</button>
          </div>
        </div>

        <!-- Клавиатура inline при édition -->
        <div v-if="editingQuery" class="results-kbd">
          <div v-for="(row, ri) in KBD_ROWS" :key="ri" class="kbd-row">
            <button
              v-for="key in row"
              :key="key"
              class="key"
              @mousedown.prevent="kbdPressResults(key)"
            >{{ key }}</button>
          </div>
          <div class="kbd-row">
            <button class="key key-fn"  @mousedown.prevent="kbdPressResults('⌫')">⌫ Стереть</button>
            <button class="key key-spc" @mousedown.prevent="kbdPressResults(' ')">ПРОБЕЛ</button>
            <button class="key key-go"  @mousedown.prevent="doSearchInline">Искать ›</button>
          </div>
        </div>

        <p class="list-count">{{ countLabel(searchResults.length, null) }}</p>
        <SermonList :sermons="searchResults" @select="openPreview($event, 's-results')" />
      </section>

      <!-- ── ПО ГОДАМ ── -->
      <section v-else-if="screen === 's-years'" class="scr">
        <p class="years-hint">Выберите год, чтобы увидеть все что есть</p>
        <div class="years-scroll">
          <div class="years-grid">
            <button
              v-for="year in years"
              :key="year"
              class="yr-card"
              @click="openYear(year)"
            >
              <span class="yr-num">{{ year }}</span>
              <span class="yr-cnt">{{ byYear(year).length }} проп.</span>
              <span class="yr-line" />
            </button>
          </div>
        </div>
      </section>

      <!-- ── СПИСОК ГОДА ── -->
      <section v-else-if="screen === 's-yr-list'" class="scr">
        <p class="list-count">{{ countLabel(yearResults.length, selectedYear) }}</p>
        <SermonList :sermons="yearResults" @select="openPreview($event, 's-yr-list')" />
      </section>

      <!-- ── ПРОСМОТР + ПЕЧАТЬ ── -->
      <section v-else-if="screen === 's-preview'" class="scr scr-preview">
        <div class="preview-area">

          <!-- PDF viewer -->
          <div class="pdf-viewer">
            <div v-if="pdfLoading" class="pdf-loading">
              <span class="pdf-loading-icon">📄</span>
              <span class="pdf-loading-text">Загрузка документа…</span>
            </div>
            <iframe
              class="pdf-frame"
              :src="pdfUrl"
              @load="pdfLoading = false"
            />
          </div>

          <!-- Панель печати -->
          <aside class="print-side">
            <div class="info-card">
              <p class="ic-label">Сведения о документе</p>
              <div class="ic-row"><span class="ic-k">Код</span>   <span class="ic-v">{{ curSermon?.code }}</span></div>
              <div class="ic-row"><span class="ic-k">Дата</span>  <span class="ic-v">{{ curSermon?.date }}</span></div>
              <div class="ic-row"><span class="ic-k">Место</span> <span class="ic-v">{{ curSermon?.lieu }}</span></div>
              <div class="ic-row"><span class="ic-k">Файл</span>  <span class="ic-v">{{ curSermon?.filename }}</span></div>
            </div>

            <div class="copies-card">
              <p class="ic-label">Количество копий</p>
              <div class="copies-ctrl">
                <button class="cop-btn" @click="printer.adjustCopies(-1)">−</button>
                <span class="cop-num">{{ printer.copies.value }}</span>
                <button class="cop-btn" @click="printer.adjustCopies(1)">+</button>
              </div>
            </div>

            <button
              class="print-btn"
              :class="printBtnClass"
              :disabled="printer.status.value === 'printing'"
              @click="doPrint"
            >
              <span class="pb-icon">{{ printBtnIcon }}</span>
              <span class="pb-label">{{ printBtnLabel }}</span>
              <span class="pb-sub">{{ printBtnSub }}</span>
            </button>
          </aside>
        </div>
      </section>

    </main>

    <!-- Toast -->
    <Transition name="toast">
      <div v-if="toastMsg" class="toast">{{ toastMsg }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import SermonList  from './components/SermonList.vue'
import { useSermons } from './composables/useSermons'
import { usePrinter } from './composables/usePrinter'
import type { Screen, Sermon } from './types'
import { getCurrentWindow } from '@tauri-apps/api/window'

/* ─── Keyboard ─────────────────────────────────────── */
const KBD_ROWS = ['ЙЦУКЕНГШЩЗХЪ', 'ФЫВАПРОЛДЖЭ', 'ЯЧСМИТЬБЮ']

/* ─── Composables ──────────────────────────────────── */
const {
  years, totalCount, yearRange,
  sourceType, source: sermonsSource,
  init, selectLocalFolder, setRemoteUrl,
  search, byYear,
} = useSermons()

const printer = usePrinter()

/* ─── Navigation ──────────────────────────────────── */
const screen     = ref<Screen>('s-setup')
const prevScreen = ref<Screen>('s-home')

const headerTitle = computed(() => {
  switch (screen.value) {
    case 's-results':  return 'Результаты поиска'
    case 's-years':    return 'По годам'
    case 's-yr-list':  return `Документы ${selectedYear.value} года`
    case 's-preview':  return curSermon.value?.title ?? ''
    default:           return ''
  }
})
const headerSub = computed(() => {
  if (screen.value === 's-yr-list')
    return countLabel(yearResults.value.length, selectedYear.value)
  if (screen.value === 's-preview' && curSermon.value)
    return `${curSermon.value.date} · ${curSermon.value.lieu}`
  return ''
})
const backLabel = computed(() => {
  switch (screen.value) {
    case 's-results': return 'Главная'
    case 's-years':   return 'Главная'
    case 's-yr-list': return 'По годам'
    default:          return 'Назад'
  }
})

function navigate(s: Screen): void {
  if (s !== 's-results') editingQuery.value = false
  screen.value = s
  if (s === 's-home') nextTick(() => searchInputRef.value?.focus())
}

function goBack(): void {
  const map: Partial<Record<Screen, Screen>> = {
    's-results':  's-home',
    's-years':    's-home',
    's-yr-list':  's-years',
    's-preview':  prevScreen.value,
  }
  navigate(map[screen.value] ?? 's-home')
}

/* ─── Setup screen ─────────────────────────────────── */
const setupTab     = ref<'local' | 'remote'>('local')
const remoteUrl    = ref('')
const loadingSource = ref(false)
const setupError   = ref('')
const currentSource = sermonsSource

async function handleSelectFolder(): Promise<void> {
  loadingSource.value = true
  setupError.value    = ''
  try {
    const path = await selectLocalFolder()
    if (path) { navigate('s-home'); showToast(`✓ ${totalCount.value} данных загружено`) }
  } catch (e: unknown) {
    setupError.value = e instanceof Error ? e.message : String(e)
  } finally { loadingSource.value = false }
}

async function handleSetRemote(): Promise<void> {
  const url = remoteUrl.value.trim()
  if (!url) return
  loadingSource.value = true
  setupError.value    = ''
  try {
    await setRemoteUrl(url)
    navigate('s-home')
    showToast(`✓ ${totalCount.value} данных загружено`)
  } catch (e: unknown) {
    setupError.value = e instanceof Error ? e.message : String(e)
  } finally { loadingSource.value = false }
}

/* ─── Home search ──────────────────────────────────── */
const query          = ref('')
const lastQuery      = ref('')
const searchResults  = ref<Sermon[]>([])
const searchFocused  = ref(false)
const searchInputRef = ref<HTMLInputElement | null>(null)
const resultsInputRef = ref<HTMLInputElement | null>(null)
const editingQuery   = ref(false)

const yearRangeLabel = computed(() =>
  totalCount.value
    ? `С ${yearRange.value} год · ${totalCount.value} данных`
    : 'Загрузка…'
)

function kbdPress(key: string): void {
  query.value = key === '⌫' ? query.value.slice(0, -1) : query.value + key
  searchInputRef.value?.focus()
}

function doSearch(): void {
  const q = query.value.trim()
  if (!q) return
  lastQuery.value     = q
  searchResults.value = search(q)
  editingQuery.value  = false
  prevScreen.value    = 's-results'
  navigate('s-results')
}

function startEditQuery(): void {
  query.value        = lastQuery.value
  editingQuery.value = true
  nextTick(() => resultsInputRef.value?.focus())
}

function cancelEditQuery(): void {
  query.value        = lastQuery.value
  editingQuery.value = false
}

function doSearchInline(): void {
  const q = query.value.trim()
  if (!q) return
  lastQuery.value     = q
  searchResults.value = search(q)
  editingQuery.value  = false
}

function kbdPressResults(key: string): void {
  query.value = key === '⌫' ? query.value.slice(0, -1) : query.value + key
  resultsInputRef.value?.focus()
}

/* ─── Years ─────────────────────────────────────────── */
const selectedYear = ref<number | null>(null)
const yearResults  = ref<Sermon[]>([])

function openYear(year: number): void {
  selectedYear.value = year
  yearResults.value  = byYear(year)
  prevScreen.value   = 's-yr-list'
  navigate('s-yr-list')
}

/* ─── Preview ──────────────────────────────────────── */
const curSermon  = ref<Sermon | null>(null)
const pdfUrl     = ref('')
const pdfLoading = ref(false)

function openPreview(sermon: Sermon, from: Screen): void {
  curSermon.value  = sermon
  prevScreen.value = from
  pdfLoading.value = true
  printer.resetCopies()
  pdfUrl.value = printer.getPdfUrl(sermonsSource.value, sermon.filename, sourceType.value)
  navigate('s-preview')
}

/* ─── Print ─────────────────────────────────────────── */
const printBtnClass = computed(() => ({
  'print-btn--printing': printer.status.value === 'printing',
  'print-btn--success':  printer.status.value === 'success',
  'print-btn--error':    printer.status.value === 'error',
}))
const printBtnIcon  = computed(() =>
  ({ printing: '⏳', success: '✓', error: '✕', idle: '🖨️' })[printer.status.value])
const printBtnLabel = computed(() =>
  ({ printing: 'ОТПРАВКА…', success: 'ОТПРАВЛЕНО!', error: 'ОШИБКА', idle: 'РАСПЕЧАТАТЬ' })[printer.status.value])
const printBtnSub = computed(() => {
  if (printer.status.value === 'error')   return printer.errorMsg.value
  if (printer.status.value === 'success') return 'Печать идёт…'
  const c = printer.copies.value
  return `${c} копи${c === 1 ? 'я' : c < 5 ? 'и' : 'й'} · Двусторонняя`
})

async function doPrint(): Promise<void> {
  if (!curSermon.value) return
  await printer.print(curSermon.value, sermonsSource.value, sourceType.value)
  if (printer.status.value === 'success') showToast(`✓ Отправлено: ${printer.copies.value} коп.`)
  else if (printer.status.value === 'error') showToast(`Ошибка: ${printer.errorMsg.value}`)
}

/* ─── Utils ─────────────────────────────────────────── */
function countLabel(n: number, year: number | null): string {
  const w = n % 10 === 1 && n % 100 !== 11 ? 'ь'
    : [2,3,4].includes(n % 10) && ![12,13,14].includes(n % 100) ? 'и' : 'ей'
  return year ? `${n} проповед${w} · ${year} год` : `Найдено: ${n} проповед${w}`
}

const toastMsg = ref('')
function showToast(msg: string): void {
  toastMsg.value = msg
  setTimeout(() => { toastMsg.value = '' }, 3500)
}

const clock = ref('--:--')
function updateClock(): void {
  const n = new Date()
  clock.value = `${n.getHours()}:${String(n.getMinutes()).padStart(2, '0')}`
}

let clockInterval: ReturnType<typeof setInterval>

/* ─── Ctrl+F4 : fermeture de secours ───────────────── */
async function handleKeydown(e: KeyboardEvent): Promise<void> {
  if (e.ctrlKey && e.key === 'F4') {
    await getCurrentWindow().destroy()
  }
}

/* ─── Lifecycle ─────────────────────────────────────── */
onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  updateClock()
  clockInterval = setInterval(updateClock, 30_000)
  const ready = await init()
  if (ready) navigate('s-home')
  else {
    if (sourceType.value === 'remote') {
      remoteUrl.value = sermonsSource.value
      setupTab.value  = 'remote'
    }
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  clearInterval(clockInterval)
})
</script>

<style scoped>
/* ── App Shell ── */
.app { width: 100%; height: 100vh; display: flex; flex-direction: column; overflow: hidden; }

/* ── Header ── */
.hdr {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 36px; height: 64px; flex-shrink: 0;
  background: #fff; border-bottom: 1.5px solid #e8e0d0;
}
.brand { display: flex; align-items: center; gap: 14px; }
.brand-cross { font-family: 'EB Garamond', serif; font-size: 28px; color: #9a7530; }
.brand-name  { font-family: 'EB Garamond', serif; font-size: 19px; color: #2a1e10; }
.brand-sub   { font-size: 9px; color: #b0a090; letter-spacing: .1em; text-transform: uppercase; margin-top: 2px; }
.hdr-center  { flex: 1; text-align: center; padding: 0 16px; min-width: 0; }
.hdr-page-title { font-family: 'EB Garamond', serif; font-size: 18px; color: #6a5d4e; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.hdr-page-sub   { font-size: 11px; color: #b0a090; margin-top: 1px; }
.hdr-right { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.hdr-clock  { font-family: 'EB Garamond', serif; font-size: 21px; color: #b0a090; }
.back-btn, .folder-btn {
  background: #f5f1ea; border: 1.5px solid #ddd5c5; border-radius: 10px;
  padding: 9px 18px; color: #6a5d4e; font-size: 13px; font-weight: 500;
  cursor: pointer; white-space: nowrap; transition: all .15s;
}
.back-btn {
  min-height: 48px;
  padding: 12px 26px;
  border-radius: 12px;
  font-size: 18px;
  font-weight: 600;
}
.back-btn:hover, .folder-btn:hover { background: #ede8df; border-color: #c9b89a; }

/* ── Screens ── */
.screens { flex: 1; overflow: hidden; position: relative; }
.scr { position: absolute; inset: 0; display: flex; flex-direction: column; background: #f0ece4; }
.scr-center { align-items: center; justify-content: center; }
.scr-preview { background: #c8bfae; }

/* ══ SETUP ══ */
.setup-box {
  background: #fff; border: 1.5px solid #e8e0d0; border-radius: 20px;
  padding: 40px 52px; max-width: 560px; width: 100%;
  display: flex; flex-direction: column; gap: 20px;
}
.setup-cross { font-family: 'EB Garamond', serif; font-size: 40px; color: #c9b89a; text-align: center; }
.setup-title { font-family: 'EB Garamond', serif; font-size: 26px; color: #2a1e10; text-align: center; }
.setup-sub   { font-size: 14px; color: #8a7d6a; text-align: center; line-height: 1.5; }

.setup-tabs { display: flex; gap: 0; border: 1.5px solid #ddd5c5; border-radius: 10px; overflow: hidden; }
.stab {
  flex: 1; padding: 11px; font-size: 14px; font-weight: 500; cursor: pointer;
  background: #faf6ef; color: #8a7d6a; border: none; transition: all .15s;
}
.stab.active { background: #9a7530; color: #fff; }
.stab:first-child { border-right: 1px solid #ddd5c5; }

.setup-section { display: flex; flex-direction: column; gap: 14px; }
.setup-hint { font-size: 13px; color: #8a7d6a; line-height: 1.6; }
.setup-hint code { background: #f5f1ea; padding: 1px 6px; border-radius: 4px; font-size: 12px; }

.setup-current {
  display: flex; align-items: center; gap: 10px;
  background: #f5f1ea; border: 1.5px solid #ddd5c5; border-radius: 10px; padding: 10px 14px;
}
.setup-current-icon { font-size: 18px; flex-shrink: 0; }
.setup-current-path { font-size: 12px; color: #6a5d4e; word-break: break-all; font-style: italic; }

.url-input-wrap { display: flex; }
.url-input {
  flex: 1; background: #fff; border: 1.5px solid #ddd5c5; border-radius: 10px;
  padding: 12px 16px; font-size: 15px; color: #2a1e10; outline: none; font-family: 'Inter', sans-serif;
  transition: border-color .2s; -webkit-user-select: text; user-select: text;
}
.url-input:focus { border-color: #9a7530; box-shadow: 0 0 0 3px rgba(154,117,48,.1); }
.url-input::placeholder { color: #ccc4b4; }

.setup-btn {
  background: #9a7530; color: #fff; border: none; border-radius: 12px;
  padding: 14px 28px; font-size: 15px; font-weight: 600; cursor: pointer;
  transition: all .15s; display: flex; align-items: center; gap: 10px; justify-content: center;
}
.setup-btn:hover:not(:disabled) { background: #7a5a20; }
.setup-btn:disabled { opacity: .6; cursor: wait; }
.setup-error { font-size: 13px; color: #c03020; background: #fff5f5; border: 1px solid #f0c8c0; border-radius: 8px; padding: 10px 14px; line-height: 1.5; }

/* ══ HOME ══ */
.home-body {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  justify-content: center; padding: 14px 28px; gap: 16px;
}
.home-title { font-family: 'EB Garamond', serif; font-size: 24px; color: #6a5d4e; font-style: italic; }
.home-title span { color: #9a7530; font-style: normal; font-weight: 500; }

.search-wrap {
  width: 100%; display: flex; background: #fff;
  border: 2px solid #ddd5c5; border-radius: 16px; overflow: hidden; transition: border-color .2s;
}
.search-wrap.focused { border-color: #9a7530; box-shadow: 0 0 0 3px rgba(154,117,48,.1); }
.sw-icon  { padding: 0 20px; display: flex; align-items: center; color: #b0a090; font-size: 22px; flex-shrink: 0; }
.sw-input {
  flex: 1; background: transparent; border: none; outline: none;
  font-family: 'EB Garamond', serif; font-size: 28px; color: #2a1e10;
  padding: 14px 0; caret-color: #9a7530; -webkit-user-select: text; user-select: text;
}
.sw-input::placeholder { color: #ccc4b4; }
.sw-clear { padding: 0 20px; font-size: 20px; color: #ccc4b4; cursor: pointer; border-left: 1px solid #ede8df; display: flex; align-items: center; transition: color .15s; }
.sw-clear:hover { color: #9a7530; }

.kbd { width: 100%; display: flex; flex-direction: column; gap: 10px; }
.kbd-row { display: flex; gap: 10px; justify-content: center; }
.key {
  background: #fff; border: 1.5px solid #ddd5c5; color: #2a1e10;
  border-radius: 12px; font-family: 'EB Garamond', serif; font-size: 31px;
  height: 78px; display: flex; align-items: center; justify-content: center;
  cursor: pointer; flex: 1; max-width: 92px; transition: all .1s;
}
.key:hover  { background: #faf6ef; border-color: #c9b89a; }
.key:active { background: #9a7530; color: #fff; border-color: #9a7530; transform: scale(.93); }
.key-fn  { max-width: 166px; font-size: 17px; font-family: 'Inter', sans-serif; color: #6a5d4e; background: #faf6ef; }
.key-fn:active { background: #9a7530; color: #fff; }
.key-spc { max-width: 520px; font-size: 16px; font-family: 'Inter', sans-serif; color: #b0a090; letter-spacing: .08em; }
.key-go  { max-width: 178px; background: #9a7530; color: #fff; border-color: #9a7530; font-size: 18px; font-family: 'Inter', sans-serif; font-weight: 600; }
.key-go:hover { background: #7a5a20; }

.yr-browse {
  width: 100%; background: #fff; border: 2px solid #ddd5c5; border-radius: 14px;
  padding: 16px 26px; display: flex; align-items: center; gap: 18px; cursor: pointer; transition: all .15s;
}
.yr-browse:hover  { background: #faf6ef; border-color: #c9b89a; }
.yr-browse:active { transform: scale(.98); }
.yrb-icon  { font-size: 26px; flex-shrink: 0; }
.yrb-body  { flex: 1; display: flex; flex-direction: column; gap: 3px; text-align: left; }
.yrb-title { font-size: 16px; font-weight: 600; color: #2a1e10; }
.yrb-sub   { font-size: 13px; color: #b0a090; }
.yrb-arr   { font-size: 24px; color: #9a7530; }

/* ══ RESULTS / QBAR ══ */
.qbar { padding: 12px 36px; background: #fff; border-bottom: 1.5px solid #e8e0d0; flex-shrink: 0; display: flex; gap: 12px; }
.qb-box { flex: 1; background: #f5f1ea; border: 1.5px solid #ddd5c5; border-radius: 12px; display: flex; align-items: center; overflow: hidden; transition: border-color .2s; }
.qb-box--editing { border-color: #9a7530; box-shadow: 0 0 0 3px rgba(154,117,48,.1); background: #fff; }
.qb-icon { padding: 0 16px; font-size: 17px; color: #b0a090; flex-shrink: 0; }
.qb-text { flex: 1; font-family: 'EB Garamond', serif; font-size: 21px; color: #9a7530; padding: 12px 0; }
.qb-input {
  flex: 1; background: transparent; border: none; outline: none;
  font-family: 'EB Garamond', serif; font-size: 21px; color: #2a1e10;
  padding: 12px 0; caret-color: #9a7530; -webkit-user-select: text; user-select: text;
}
.qb-clear { border-left: 1px solid #ede8df; }
.qb-mod  { padding: 0 18px; font-size: 13px; font-weight: 500; color: #8a7d6a; border-left: 1.5px solid #ddd5c5; cursor: pointer; display: flex; align-items: center; background: transparent; border-radius: 0; border-top: none; border-bottom: none; border-right: none; transition: all .15s; white-space: nowrap; height: 100%; }
.qb-mod:hover { background: #ede8df; color: #9a7530; }
.qb-mod--cancel { color: #c03020; }
.qb-mod--cancel:hover { background: #fff5f5; color: #c03020; }
.results-kbd { padding: 14px 18px 14px; background: #fff; border-bottom: 1.5px solid #e8e0d0; flex-shrink: 0; display: flex; flex-direction: column; gap: 10px; }
.list-count { padding: 10px 36px 4px; font-size: 13px; color: #b0a090; flex-shrink: 0; }

/* ══ YEARS ══ */
.years-hint   { padding: 14px 36px 6px; font-family: 'EB Garamond', serif; font-size: 16px; color: #b0a090; font-style: italic; flex-shrink: 0; }
.years-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 10px 20px 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.years-grid {
  width: 100%;
  max-width: none;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 20px;
}
.yr-card {
  background: #fff; border: 1.5px solid #e8e0d0; border-radius: 14px;
  min-height: 170px; padding: 28px 14px; text-align: center; cursor: pointer; transition: all .15s;
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; width: 100%;
}
.yr-card:hover  { border-color: #c9b89a; background: #fdfaf5; }
.yr-card:active { border-color: #9a7530; transform: scale(.97); }
.yr-num  { font-family: 'EB Garamond', serif; font-size: 54px; line-height: 1; color: #2a1e10; }
.yr-cnt  { font-size: 15px; color: #b0a090; }
.yr-line { width: 44px; height: 4px; background: #e8e0d0; border-radius: 2px; }
.yr-card:hover .yr-line { background: #c9b89a; }

/* ══ PREVIEW ══ */
.preview-area { flex: 1; display: flex; overflow: hidden; padding: 16px 20px 18px; gap: 16px; }
.pdf-viewer   { flex: 1; border-radius: 4px; overflow: hidden; box-shadow: 0 6px 36px rgba(0,0,0,.22); background: #fff; position: relative; }
.pdf-frame    { width: 100%; height: 100%; border: none; display: block; }
.pdf-loading  { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 14px; background: #fffef8; z-index: 1; }
.pdf-loading-icon { font-size: 40px; animation: spin 1.5s linear infinite; }
.pdf-loading-text { font-family: 'EB Garamond', serif; font-size: 18px; color: #b0a090; font-style: italic; }
@keyframes spin { to { transform: rotate(360deg); } }

.print-side { width: 250px; display: flex; flex-direction: column; gap: 12px; flex-shrink: 0; }
.info-card, .copies-card { background: #fff; border: 1.5px solid #ddd5c5; border-radius: 14px; padding: 16px 18px; }
.ic-label { font-size: 9px; letter-spacing: .14em; text-transform: uppercase; color: #b0a090; margin-bottom: 10px; }
.ic-row   { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 7px; }
.ic-row:last-child { margin-bottom: 0; }
.ic-k     { font-size: 12px; color: #b0a090; }
.ic-v     { font-size: 12px; color: #2a1e10; font-weight: 500; text-align: right; max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ic-v--gold { color: #9a7530; }
.copies-ctrl { display: flex; align-items: stretch; border: 1.5px solid #e8e0d0; border-radius: 12px; overflow: hidden; margin-top: 10px; }
.cop-btn { width: 50px; display: flex; align-items: center; justify-content: center; font-size: 26px; color: #6a5d4e; cursor: pointer; background: #f5f1ea; border: none; transition: all .15s; flex-shrink: 0; }
.cop-btn:hover  { background: #ede8df; }
.cop-btn:active { background: #9a7530; color: #fff; }
.cop-num { flex: 1; font-family: 'EB Garamond', serif; font-size: 30px; color: #2a1e10; text-align: center; padding: 9px 0; border-left: 1.5px solid #e8e0d0; border-right: 1.5px solid #e8e0d0; }

.print-btn {
  width: 100%; background: #9a7530; border: none; border-radius: 14px;
  padding: 20px 16px; cursor: pointer; display: flex; flex-direction: column;
  align-items: center; gap: 7px; transition: all .15s; margin-top: auto;
}
.print-btn:hover:not(:disabled) { background: #7a5a20; }
.print-btn:active:not(:disabled) { transform: scale(.97); }
.print-btn:disabled  { cursor: wait; }
.print-btn--printing { background: #c0902a; }
.print-btn--success  { background: #2d6a1f; }
.print-btn--error    { background: #8a2020; }
.pb-icon  { font-size: 32px; line-height: 1; }
.pb-label { font-size: 20px; font-weight: 700; color: #fff; letter-spacing: .04em; }
.pb-sub   {
  max-width: 100%;
  font-size: 10px;
  color: rgba(255,255,255,.55);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ══ TOAST ══ */
.toast {
  position: fixed; bottom: 28px; left: 50%; transform: translateX(-50%);
  background: #2a1e10; color: #fff; padding: 11px 22px; border-radius: 10px;
  font-size: 14px; z-index: 999; white-space: nowrap;
}
.toast-enter-active, .toast-leave-active { transition: opacity .3s; }
.toast-enter-from,  .toast-leave-to      { opacity: 0; }
</style>
