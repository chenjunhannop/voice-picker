import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PlaybackProgress {
  current_secs: number
  total_secs: number
  percent: number
}

export interface TtsResponse {
  success: boolean
  audio_data: number[] | null
  error: string | null
}

export type PlaybackState = 'Idle' | 'Playing' | 'Paused' | 'Stopped'

export function useAudioPlayer() {
  const isPlaying = ref(false)
  const isPaused = ref(false)
  const state = ref<PlaybackState>('Idle')
  const progress = ref<PlaybackProgress>({ current_secs: 0, total_secs: 0, percent: 0 })

  // 进度轮询定时器
  let pollingInterval: number | null = null

  // 开始轮询播放进度
  const startPolling = () => {
    stopPolling()
    pollingInterval = window.setInterval(async () => {
      try {
        const progressData = await invoke<PlaybackProgress>('get_playback_progress')
        progress.value = progressData

        // 根据进度更新状态
        if (progressData.percent >= 100) {
          isPlaying.value = false
          isPaused.value = false
          state.value = 'Idle'
          stopPolling()
        }
      } catch (error) {
        console.error('获取播放进度失败:', error)
      }
    }, 500) // 每 500ms 轮询一次
  }

  // 停止轮询
  const stopPolling = () => {
    if (pollingInterval) {
      clearInterval(pollingInterval)
      pollingInterval = null
    }
  }

  // 播放音频（先合成 TTS 再播放）
  const play = async (text: string, speed?: number, volume?: number) => {
    try {
      isPlaying.value = true
      isPaused.value = false
      state.value = 'Playing'

      // 1. 先调用 TTS 合成
      const result = await invoke<TtsResponse>('synthesize_tts', {
        request: { text, speed, volume }
      })

      // 2. 合成成功后播放
      if (result.success) {
        await invoke('play_audio')
        // 开始轮询进度
        startPolling()
        return true
      } else {
        throw new Error(result.error || 'TTS 合成失败')
      }
    } catch (error) {
      console.error('播放失败:', error)
      isPlaying.value = false
      state.value = 'Idle'
      throw error
    }
  }

  // 暂停播放
  const pause = async () => {
    try {
      await invoke('pause_audio')
      isPlaying.value = false
      isPaused.value = true
      state.value = 'Paused'
      stopPolling()
    } catch (error) {
      console.error('暂停失败:', error)
      throw error
    }
  }

  // 继续播放
  const resume = async () => {
    try {
      await invoke('resume_audio')
      isPlaying.value = true
      isPaused.value = false
      state.value = 'Playing'
      startPolling()
    } catch (error) {
      console.error('继续失败:', error)
      throw error
    }
  }

  // 停止播放
  const stop = async () => {
    try {
      await invoke('stop_audio')
      isPlaying.value = false
      isPaused.value = false
      state.value = 'Idle'
      progress.value = { current_secs: 0, total_secs: 0, percent: 0 }
      stopPolling()
    } catch (error) {
      console.error('停止失败:', error)
      throw error
    }
  }

  // 跳转到指定位置（秒）
  const seek = async (position: number) => {
    try {
      await invoke('seek_audio', { position })
      progress.value.current_secs = position
    } catch (error) {
      console.error('跳转失败:', error)
      throw error
    }
  }

  // 格式化时间为 mm:ss
  const formatTime = (secs: number): string => {
    const minutes = Math.floor(secs / 60)
    const seconds = Math.floor(secs % 60)
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  }

  // 清理
  onUnmounted(() => {
    stopPolling()
  })

  return {
    // 状态
    isPlaying,
    isPaused,
    state,
    progress,

    // 方法
    play,
    pause,
    resume,
    stop,
    seek,
    formatTime,
  }
}
