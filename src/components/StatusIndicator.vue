<template>
  <div class="status-indicator" :class="statusClass">
    <div class="status-dot"></div>
    <span>{{ displayText }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'

type Status = 'idle' | 'loading' | 'playing' | 'paused' | 'error' | 'success'

const status = ref<Status>('idle')
const displayText = ref('就绪')

const statusClass = computed(() => {
  const classes = {
    idle: 'status-idle',
    loading: 'status-loading',
    playing: 'status-playing',
    paused: 'status-paused',
    error: 'status-error',
    success: 'status-success'
  }
  return classes[status.value]
})

const setStatus = (newStatus: Status, text: string) => {
  status.value = newStatus
  displayText.value = text
}

onMounted(() => {
  // 监听全局状态事件
  listen('tts-status', (event) => {
    const payload = event.payload as string
    displayText.value = payload

    if (payload.includes('正在')) {
      status.value = 'loading'
    } else if (payload.includes('播放')) {
      status.value = 'playing'
    } else if (payload.includes('错误')) {
      status.value = 'error'
    } else if (payload.includes('就绪') || payload.includes('完成')) {
      status.value = payload.includes('完成') ? 'success' : 'idle'
    }
  })
})

defineExpose({ setStatus })
</script>

<style scoped>
.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 14px;
  transition: all 0.3s;
}

.status-idle {
  background: #e7f3ff;
  color: #0066cc;
}

.status-loading {
  background: #fff3e0;
  color: #ff9800;
}

.status-playing {
  background: #e7f7e7;
  color: #28a745;
}

.status-paused {
  background: #fff8e1;
  color: #ffc107;
}

.status-error {
  background: #f7e7e7;
  color: #dc3545;
}

.status-success {
  background: #e8f5e9;
  color: #4caf50;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.status-loading .status-dot,
.status-playing .status-dot {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
