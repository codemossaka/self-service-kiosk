<template>
    <div class="ser-list">
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
  </template>
  
  <script setup lang="ts">
  import type { Sermon } from '../types'
  
  defineProps<{ sermons: Sermon[] }>()
  defineEmits<{ select: [s: Sermon] }>()
  </script>
  
  <style scoped>
  .ser-list {
    flex: 1; overflow-y: auto;
    padding: 8px 24px 20px;
    display: flex; flex-direction: column; gap: 8px;
  }
  .ser-list::-webkit-scrollbar { width: 6px; }
  .ser-list::-webkit-scrollbar-thumb { background: #ddd5c5; border-radius: 3px; }
  
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