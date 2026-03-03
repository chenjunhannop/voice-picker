<template>
  <div class="audio-player-window" :class="{ collapsed: isCollapsed }">
    <!-- 窗口头部 -->
    <div class="player-header">
      <span class="player-title">正在播放</span>
      <div class="header-actions">
        <button @click="toggleCollapse" class="action-btn" title="折叠">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path fill="currentColor" :d="collapseIconPath"/>
          </svg>
        </button>
        <button @click="closeWindow" class="action-btn" title="关闭">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 窗口内容 -->
    <div class="player-content" v-show="!isCollapsed">
      <!-- 音频信息 -->
      <div class="audio-info">
        <div class="audio-text">{{ currentText }}</div>
      </div>

      <!-- 进度条 -->
      <div class="progress-section">
        <div class="progress-bar" @click="handleSeek">
          <div class="progress-fill" :style="{ width: progress.percent + '%' }"></div>
          <div class="progress-thumb" :style="{ left: progress.percent + '%' }"></div>
        </div>
        <div class="time-display">
          <span>{{ formatTime(progress.current_secs) }}</span>
          <span>{{ formatTime(progress.total_secs) }}</span>
        </div>
      </div>

      <!-- 控制按钮 -->
      <div class="controls-section">
        <button
          @click="handleStop"
          :disabled="!isPlaying && !isPaused"
          class="control-btn"
          title="停止"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M6 6h12v12H6z"/>
          </svg>
        </button>

        <button
          @click="handlePrevious"
          :disabled="!hasPrevious"
          class="control-btn"
          title="上一个"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
          </svg>
        </button>

        <button
          @click="handlePlayPause"
          :disabled="!hasAudio"
          class="control-btn primary"
          :title="isPlaying ? '暂停' : '播放'"
        >
          <svg v-if="isPlaying" viewBox="0 0 24 24" width="24" height="24">
            <path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="24" height="24">
            <path fill="currentColor" d="M8 5v14l11-7z"/>
          </svg>
        </button>

        <button
          @click="handleResume"
          :disabled="!isPaused"
          class="control-btn"
          title="继续"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M8 5v14l11-7z"/>
          </svg>
        </button>

        <button
          @click="handleNext"
          :disabled="!hasNext"
          class="control-btn"
          title="下一个"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path fill="currentColor" d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
          </svg>
        </button>
      </div>

      <!-- 速度控制 -->
      <div class="speed-section">
        <span class="speed-label">速度:</span>
        <div class="speed-buttons">
          <button
            v-for="speed in speedOptions"
            :key="speed"
            @click="setSpeed(speed)"
            :class="{ active: currentSpeed === speed }"
            class="speed-btn"
          >
            {{ speed }}x
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAudioPlayer } from '../composables/useAudioPlayer'

const { isPlaying, isPaused, progress, pause, resume, stop, formatTime } = useAudioPlayer()

// 折叠状态
const isCollapsed = ref(false)
const hasAudio = ref(false)
const hasPrevious = ref(false)
const hasNext = ref(false)
const currentText = ref('')

// 速度控制
const currentSpeed = ref(1.0)
const speedOptions = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]

// 折叠图标路径
const collapseIconPath = computed(() =>
  isCollapsed.value
    ? 'M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z'
    : 'M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6-6 6z'
)

// 折叠/展开
const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value
}

// 关闭窗口
const closeWindow = () => {
  // TODO: 隐藏浮窗
  isCollapsed.value = true
}

// 播放/暂停切换
const handlePlayPause = async () => {
  if (isPlaying.value) {
    await pause()
  } else if (isPaused.value) {
    await resume()
  }
}

// 继续播放
const handleResume = async () => {
  await resume()
}

// 停止播放
const handleStop = async () => {
  await stop()
}

// 上一个
const handlePrevious = async () => {
  // TODO: 实现上一个音频播放
  console.log('播放上一个音频')
}

// 下一个
const handleNext = async () => {
  // TODO: 实现下一个音频播放
  console.log('播放下一个音频')
}

// 跳转进度
const handleSeek = async (event: MouseEvent) => {
  const bar = event.currentTarget as HTMLElement
  const rect = bar.getBoundingClientRect()
  const percent = (event.clientX - rect.left) / rect.width
  const position = progress.value.total_secs * percent
  await invoke('seek_audio', { position })
}

// 设置速度
const setSpeed = async (speed: number) => {
  currentSpeed.value = speed
  // TODO: 更新 TTS 速度设置
  await invoke('save_settings', {
    settings: { tts_speed: speed }
  })
}

// 清理
onUnmounted(() => {
  // 清理逻辑
})
</script>

<style scoped>
.audio-player-window {
  position: fixed;
  top: 80px;
  right: 20px;
  width: 320px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  overflow: hidden;
  transition: all 0.3s ease;
}

.audio-player-window.collapsed {
  width: auto;
}

.player-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.player-title {
  font-size: 14px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.player-content {
  padding: 16px;
}

.audio-info {
  margin-bottom: 16px;
}

.audio-text {
  font-size: 13px;
  color: #333;
  line-height: 1.5;
  max-height: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.progress-section {
  margin-bottom: 16px;
}

.progress-bar {
  height: 6px;
  background: #e0e0e0;
  border-radius: 3px;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.progress-bar:hover {
  background: #d0d0d0;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea, #764ba2);
  transition: width 0.1s;
}

.progress-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 14px;
  height: 14px;
  background: white;
  border: 2px solid #667eea;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transition: left 0.1s;
}

.time-display {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
  font-size: 11px;
  color: #999;
}

.controls-section {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.control-btn {
  width: 40px;
  height: 40px;
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

.control-btn.primary {
  width: 50px;
  height: 50px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}

.control-btn.primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #5a6fd6, #6a4190);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.speed-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.speed-label {
  font-size: 12px;
  color: #666;
}

.speed-buttons {
  display: flex;
  gap: 4px;
}

.speed-btn {
  padding: 4px 8px;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  background: white;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
}

.speed-btn:hover {
  background: #f5f5f5;
}

.speed-btn.active {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  border-color: transparent;
}
</style>
