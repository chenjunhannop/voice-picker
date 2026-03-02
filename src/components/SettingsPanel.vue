<template>
  <div class="settings-panel">
    <h2>设置</h2>

    <div class="setting-group">
      <h3>快捷键</h3>
      <div class="setting-item">
        <label>朗读快捷键</label>
        <input
          type="text"
          v-model="shortcut"
          @blur="saveShortcut"
          placeholder="Cmd+Option+X"
          class="shortcut-input"
        />
        <p class="hint">支持组合键，如：Cmd+Shift+L</p>
      </div>
    </div>

    <div class="setting-group">
      <h3>语音设置</h3>
      <div class="setting-item">
        <label>语速：{{ speed.toFixed(1) }}x</label>
        <input
          type="range"
          v-model="speed"
          @input="saveSettings"
          min="0.5"
          max="2"
          step="0.1"
          class="slider"
        />
      </div>

      <div class="setting-item">
        <label>音量：{{ Math.round(volume * 100) }}%</label>
        <input
          type="range"
          v-model="volume"
          @input="saveSettings"
          min="0"
          max="1"
          step="0.1"
          class="slider"
        />
      </div>
    </div>

    <div class="setting-group">
      <h3>模型设置</h3>
      <div class="setting-item">
        <label>模型路径</label>
        <div class="path-input">
          <input
            type="text"
            v-model="modelPath"
            placeholder="models/qwen3-tts-0.6b"
            readonly
          />
          <button @click="selectModelPath" class="btn">选择</button>
        </div>
      </div>
    </div>

    <div class="actions">
      <button @click="resetSettings" class="btn btn-secondary">重置设置</button>
      <button @click="checkModel" class="btn btn-primary">检查模型</button>
    </div>

    <div v-if="message" :class="['message', messageType]">
      {{ message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const shortcut = ref('Cmd+Option+X')
const speed = ref(1.0)
const volume = ref(1.0)
const modelPath = ref('models/qwen3-tts-0.6b')
const message = ref('')
const messageType = ref<'info' | 'success' | 'error'>('info')

// 从后端加载设置
const loadSettings = async () => {
  try {
    const settings = await invoke<AppSettings>('get_settings')
    shortcut.value = settings.global_hotkey
    speed.value = settings.tts_speed
    volume.value = settings.tts_volume
  } catch (e) {
    console.error('加载设置失败:', e)
    showMessage('加载设置失败：' + e, 'error')
  }
}

// 保存快捷键
const saveShortcut = async () => {
  try {
    await invoke('save_settings', {
      settings: {
        tts_speed: speed.value,
        tts_volume: volume.value,
        global_hotkey: shortcut.value,
        auto_play: true,
        theme: 'system'
      }
    })
    showMessage('快捷键已保存', 'success')
  } catch (e) {
    console.error('保存快捷键失败:', e)
    showMessage('保存快捷键失败：' + e, 'error')
  }
}

// 保存到后端
const saveSettings = async () => {
  try {
    await invoke('save_settings', {
      settings: {
        tts_speed: speed.value,
        tts_volume: volume.value,
        global_hotkey: shortcut.value,
        auto_play: true,
        theme: 'system'
      }
    })
    showMessage('设置已保存', 'success')
  } catch (e) {
    console.error('保存设置失败:', e)
    showMessage('保存设置失败：' + e, 'error')
  }
}

// 选择模型路径
const selectModelPath = async () => {
  try {
    // 调用系统文件选择对话框
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择模型文件夹'
    })
    if (selected) {
      modelPath.value = selected.toString()
      showMessage(`模型路径已更新：${modelPath.value}`, 'success')
    }
  } catch (e) {
    showMessage('选择模型路径失败：' + e, 'error')
  }
}

// 重置设置
const resetSettings = async () => {
  try {
    await invoke('reset_settings')
    shortcut.value = 'Cmd+Option+X'
    speed.value = 1.0
    volume.value = 1.0
    showMessage('设置已重置', 'success')
  } catch (e) {
    showMessage('重置设置失败：' + e, 'error')
  }
}

// 检查模型
const checkModel = async () => {
  try {
    // 简化检查：只显示提示信息
    showMessage(`当前模型路径：${modelPath.value}\n请确保该路径存在`, 'info')
  } catch (e) {
    showMessage('模型检查失败：' + e, 'error')
  }
}

// 显示消息
const showMessage = (msg: string, type: 'info' | 'success' | 'error') => {
  message.value = msg
  messageType.value = type
  setTimeout(() => {
    message.value = ''
  }, 3000)
}

// 类型定义
interface AppSettings {
  tts_speed: number
  tts_volume: number
  global_hotkey: string
  auto_play: boolean
  theme: string
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.settings-panel {
  padding: 20px;
  max-width: 500px;
  margin: 0 auto;
}

h2 {
  margin-bottom: 20px;
  color: #333;
}

h3 {
  margin: 15px 0 10px;
  color: #666;
  font-size: 14px;
}

.setting-group {
  margin-bottom: 20px;
  padding: 15px;
  background: #f9f9f9;
  border-radius: 8px;
}

.setting-item {
  margin-bottom: 15px;
}

.setting-item label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: #555;
}

.shortcut-input {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  font-family: monospace;
}

.slider {
  width: 100%;
  margin-top: 5px;
}

.hint {
  margin-top: 5px;
  font-size: 12px;
  color: #999;
}

.path-input {
  display: flex;
  gap: 10px;
}

.path-input input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.message {
  margin-top: 15px;
  padding: 10px;
  border-radius: 4px;
  text-align: center;
}

.message.info {
  background: #e7f3ff;
  color: #0066cc;
}

.message.success {
  background: #e7f7e7;
  color: #28a745;
}

.message.error {
  background: #f7e7e7;
  color: #dc3545;
}
</style>
