<template>
  <div class="ser-list-wrap">
    <button
      class="scroll-btn scroll-btn--up"
      type="button"
      aria-label="Прокрутить вверх"
      :disabled="!sermons.length"
      @click="scrollList(-1)"
    >
      ↑
    </button>
    <div ref="listRef" class="ser-list">
      <div v-if="!sermons.length" class="empty-msg">
        Ни одной проповеди не найдено.<br>Попробуйте другое слово.
      </div>
      <button
        v-for="s in sermons"
        :key="s.code"
        class="ser-card"
        @click="$emit('select', s)"
      >
        <div class="sc-badge">
          <span class="sc-badge-lbl">Год</span>
          <span class="sc-badge-yr">{{ s.year }}</span>
        </div>
        <div class="sc-body">
          <div class="sc-title">{{ s.title }}</div>
          <div class="sc-meta">
            <span>{{ s.date }}</span>
            <span class="sc-dot" />
            <span>{{ s.lieu }}</span>
            <span class="sc-dot" />
            <span class="sc-file">{{ s.filename }}</span>
          </div>
        </div>
        <span class="sc-arr">›</span>
      </button>
    </div>
    <button
      class="scroll-btn scroll-btn--down"
      type="button"
      aria-label="Прокрутить вниз"
      :disabled="!sermons.length"
      @click="scrollList(1)"
    >
      ↓
    </button>
  </div>
  </template>
  
  <script setup lang="ts">
  import { ref } from 'vue'
  import type { Sermon } from '../types'
  
  defineProps<{ sermons: Sermon[] }>()
  defineEmits<{ select: [s: Sermon] }>()

  const listRef = ref<HTMLElement | null>(null)

  function scrollList(direction: -1 | 1): void {
    const list = listRef.value
    if (!list) return

    list.scrollBy({
      top: direction * Math.max(180, list.clientHeight * 0.75),
      behavior: 'smooth',
    })
  }
  </script>
  
  <style scoped>
  .ser-list-wrap {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 78px;
    grid-template-rows: 1fr 1fr;
    gap: 10px;
    padding: 8px 16px 20px 24px;
  }

  .ser-list {
    grid-row: 1 / 3;
    min-height: 0;
    overflow-y: auto;
    padding-right: 10px;
    display: flex; flex-direction: column; gap: 8px;
    scrollbar-width: auto;
    scrollbar-color: #c9b89a #f5f1ea;
  }
  .ser-list::-webkit-scrollbar { width: 22px; }
  .ser-list::-webkit-scrollbar-track { background: #f5f1ea; border-radius: 12px; }
  .ser-list::-webkit-scrollbar-thumb {
    background: #c9b89a;
    border: 4px solid #f5f1ea;
    border-radius: 12px;
  }

  .scroll-btn {
    width: 78px;
    min-height: 118px;
    border: 1.5px solid #ddd5c5;
    border-radius: 14px;
    background: #fff;
    color: #9a7530;
    font-size: 36px;
    font-weight: 600;
    cursor: pointer;
    transition: all .15s;
  }
  .scroll-btn:hover:not(:disabled) { background: #faf6ef; border-color: #c9b89a; }
  .scroll-btn:active:not(:disabled) { background: #9a7530; border-color: #9a7530; color: #fff; transform: scale(.97); }
  .scroll-btn:disabled { opacity: .35; cursor: default; }
  .scroll-btn--up { grid-column: 2; grid-row: 1; }
  .scroll-btn--down { grid-column: 2; grid-row: 2; }
  
  .empty-msg {
    text-align: center; padding: 60px 20px;
    font-family: 'EB Garamond', serif; font-size: 21px;
    color: #b0a090; font-style: italic; line-height: 1.7;
  }
  
  .ser-card {
    background: #fff; border: 1.5px solid #e8e0d0; border-radius: 16px;
    padding: 18px 22px; display: flex; align-items: center; gap: 18px;
    cursor: pointer; transition: all .15s; width: 100%; text-align: left;
  }
  .ser-card:hover  { border-color: #c9b89a; background: #fdfaf5; }
  .ser-card:active { border-color: #9a7530; }
  
  .sc-badge {
    width: 58px; height: 58px; background: #faf6ef;
    border: 1.5px solid #e8e0d0; border-radius: 12px;
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; flex-shrink: 0;
  }
  .sc-badge-lbl { font-size: 8px; color: #b0a090; text-transform: uppercase; letter-spacing: .08em; }
  .sc-badge-yr  { font-family: 'EB Garamond', serif; font-size: 22px; color: #9a7530; font-weight: 500; }
  
  .sc-body  { flex: 1; min-width: 0; }
  .sc-title { font-family: 'EB Garamond', serif; font-size: 22px; color: #2a1e10; line-height: 1.2; margin-bottom: 5px; }
  .sc-meta  { font-size: 12px; color: #b0a090; display: flex; flex-wrap: wrap; gap: 4px 12px; align-items: center; }
  .sc-file  { color: #c8c0b0; font-style: italic; }
  .sc-dot   { width: 3px; height: 3px; border-radius: 50%; background: #ddd5c5; display: inline-block; flex-shrink: 0; }
  
  .sc-arr { font-size: 28px; color: #ddd5c5; flex-shrink: 0; transition: color .15s; }
  .ser-card:hover .sc-arr { color: #9a7530; }
  </style>
