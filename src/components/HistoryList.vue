<template>
  <div class="history-list">
    <h3>朗读历史</h3>

    <div v-if="history.length === 0" class="empty-state">
      <p>暂无历史记录</p>
    </div>

    <div v-else class="history-items">
      <div
        v-for="item in history"
        :key="item.id"
        class="history-item"
        @click="replay(item)"
      >
        <div class="history-content">
          <span class="text">{{ item.text.substring(0, 50) }}{{ item.text.length > 50 ? '...' : '' }}</span>
          <span class="time">{{ formatTime(item.timestamp) }}</span>
        </div>
        <button class="replay-btn" title="重新播放">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path fill="currentColor" d="M8 5v14l11-7z"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="actions" v-if="history.length > 0">
      <button @click="clearHistory" class="btn btn-secondary">清空历史</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface HistoryItem {
  id: number
  text: string
  timestamp: number
  audioData?: Uint8Array
}

const history = ref<HistoryItem[]>([])

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
  return date.toLocaleDateString('zh-CN')
}

const replay = async (item: HistoryItem) => {
  try {
    if (item.audioData) {
      // 如果有音频数据，直接播放
      await invoke('play_audio')
    } else {
      // 否则重新生成 TTS
      const speed = parseFloat(localStorage.getItem('tts-speed') || '1.0')
      const volume = parseFloat(localStorage.getItem('tts-volume') || '1.0')
      await invoke('synthesize_tts', {
        request: {
          text: item.text,
          speed,
          volume
        }
      })
      await invoke('play_audio')
    }
  } catch (e) {
    console.error('重新播放失败:', e)
    alert('重新播放失败：' + e)
  }
}

const clearHistory = () => {
  if (confirm('确定要清空历史记录吗？')) {
    history.value = []
    localStorage.removeItem('tts-history')
  }
}

onMounted(() => {
  // 从本地存储加载历史
  const saved = localStorage.getItem('tts-history')
  if (saved) {
    try {
      history.value = JSON.parse(saved)
    } catch (e) {
      console.error('加载历史失败:', e)
    }
  }
})

// 暴露添加历史的方法
defineExpose({
  addHistory: (item: HistoryItem) => {
    history.value.unshift(item)
    // 只保留最近的 50 条
    if (history.value.length > 50) {
      history.value = history.value.slice(0, 50)
    }
    localStorage.setItem('tts-history', JSON.stringify(history.value))
  }
})
</script>

<style scoped>
.history-list {
  padding: 20px;
  max-width: 500px;
  margin: 0 auto;
}

h3 {
  margin-bottom: 15px;
  color: #333;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #999;
}

.history-items {
  max-height: 400px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  margin-bottom: 8px;
  background: #f9f9f9;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.history-item:hover {
  background: #f0f0f0;
}

.history-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.text {
  font-size: 14px;
  color: #333;
}

.time {
  font-size: 12px;
  color: #999;
}

.replay-btn {
  padding: 8px;
  border: none;
  background: #e0e0e0;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-left: 10px;
}

.replay-btn:hover {
  background: #d0d0d0;
}

.actions {
  margin-top: 15px;
  text-align: center;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}
</style>
