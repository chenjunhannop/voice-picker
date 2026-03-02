<template>
  <div class="player-controls">
    <div class="status-indicator" :class="statusClass">
      <div class="status-dot"></div>
      <span>{{ statusText }}</span>
    </div>

    <div class="controls">
      <button
        @click="handlePlay"
        :disabled="!hasAudio || isPlaying"
        class="control-btn"
        title="播放"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path fill="currentColor" d="M8 5v14l11-7z"/>
        </svg>
      </button>

      <button
        @click="handlePause"
        :disabled="!isPlaying"
        class="control-btn"
        title="暂停"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
        </svg>
      </button>

      <button
        @click="handleStop"
        :disabled="!isPlaying && !isPaused"
        class="control-btn"
        title="停止"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path fill="currentColor" d="M6 6h12v12H6z"/>
        </svg>
      </button>
    </div>

    <div class="progress" v-if="showProgress">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
      </div>
      <span class="progress-text">{{ progressText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

type PlayerStatus = 'idle' | 'playing' | 'paused' | 'error'

const status = ref<PlayerStatus>('idle')
const hasAudio = ref(false)
const isPlaying = ref(false)
const isPaused = ref(false)
const progressPercent = ref(0)

const statusClass = computed(() => `status-${status.value}`)
const statusText = computed(() => {
  const texts = {
    idle: '就绪',
    playing: '播放中',
    paused: '已暂停',
    error: '错误'
  }
  return texts[status.value]
})

const showProgress = computed(() => status.value === 'playing' || status.value === 'paused')
const progressText = computed(() => `${Math.round(progressPercent.value)}%`)

const handlePlay = async () => {
  try {
    await invoke('play_audio')
    status.value = 'playing'
    isPlaying.value = true
    isPaused.value = false
  } catch (e) {
    console.error('播放失败:', e)
    status.value = 'error'
    isPlaying.value = false
  }
}

const handlePause = async () => {
  try {
    await invoke('pause_audio')
    status.value = 'paused'
    isPlaying.value = false
    isPaused.value = true
  } catch (e) {
    console.error('暂停失败:', e)
  }
}

const handleStop = async () => {
  try {
    await invoke('stop_audio')
    status.value = 'idle'
    progressPercent.value = 0
    isPlaying.value = false
    isPaused.value = false
  } catch (e) {
    console.error('停止失败:', e)
  }
}

onMounted(() => {
  // 监听状态更新
  listen('tts-status', (event) => {
    const payload = event.payload as string
    if (payload.includes('播放')) {
      status.value = 'playing'
      hasAudio.value = true
      isPlaying.value = true
      isPaused.value = false
    } else if (payload.includes('错误')) {
      status.value = 'error'
      isPlaying.value = false
    } else if (payload.includes('就绪')) {
      status.value = 'idle'
      isPlaying.value = false
      isPaused.value = false
    } else if (payload.includes('暂停')) {
      status.value = 'paused'
      isPlaying.value = false
      isPaused.value = true
    }
  })
})
</script>

<style scoped>
.player-controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 20px;
  margin-bottom: 20px;
  font-size: 14px;
}

.status-idle {
  background: #e7f3ff;
  color: #0066cc;
}

.status-playing {
  background: #e7f7e7;
  color: #28a745;
}

.status-paused {
  background: #fff3e0;
  color: #ff9800;
}

.status-error {
  background: #f7e7e7;
  color: #dc3545;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.controls {
  display: flex;
  gap: 12px;
}

.control-btn {
  width: 50px;
  height: 50px;
  border: none;
  border-radius: 50%;
  background: #f0f0f0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.control-btn:hover:not(:disabled) {
  background: #e0e0e0;
  transform: scale(1.05);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.progress {
  width: 100%;
  margin-top: 20px;
}

.progress-bar {
  height: 4px;
  background: #e0e0e0;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #007bff;
  transition: width 0.3s;
}

.progress-text {
  display: block;
  text-align: center;
  margin-top: 8px;
  font-size: 12px;
  color: #666;
}
</style>
