<template>
  <div class="app">
    <header class="header">
      <h1>VoicePicker</h1>
      <StatusIndicator ref="statusIndicator" />
    </header>

    <main class="main">
      <div class="tabs">
        <button
          :class="['tab', { active: activeTab === 'player' }]"
          @click="activeTab = 'player'"
        >
          播放控制
        </button>
        <button
          :class="['tab', { active: activeTab === 'settings' }]"
          @click="activeTab = 'settings'"
        >
          设置
        </button>
        <button
          :class="['tab', { active: activeTab === 'history' }]"
          @click="activeTab = 'history'"
        >
          历史记录
        </button>
      </div>

      <div class="tab-content">
        <PlayerControls v-show="activeTab === 'player'" />
        <SettingsPanel v-show="activeTab === 'settings'" />
        <HistoryList v-show="activeTab === 'history'" ref="historyList" />
      </div>
    </main>

    <footer class="footer">
      <p>快捷键：<kbd>Cmd</kbd> + <kbd>Option</kbd> + <kbd>X</kbd> 选中文本后朗读</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import StatusIndicator from './components/StatusIndicator.vue'
import PlayerControls from './components/PlayerControls.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import HistoryList from './components/HistoryList.vue'

const activeTab = ref<'player' | 'settings' | 'history'>('player')
const statusIndicator = ref<InstanceType<typeof StatusIndicator> | null>(null)
const historyList = ref<InstanceType<typeof HistoryList> | null>(null)

// 监听 TTS 状态事件
listen('tts-status', (event) => {
  const payload = event.payload as string
  console.log('Status update:', payload)
})

// 监听朗读完成，添加历史记录
listen('tts-complete', (event) => {
  const payload = event.payload as { text: string; timestamp: number }
  if (historyList.value) {
    historyList.value.addHistory({
      id: Date.now(),
      text: payload.text,
      timestamp: payload.timestamp
    })
  }
})
</script>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.main {
  flex: 1;
  overflow-y: auto;
}

.tabs {
  display: flex;
  border-bottom: 1px solid #e0e0e0;
  background: #fafafa;
}

.tab {
  flex: 1;
  padding: 12px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: #666;
  transition: all 0.2s;
  border-bottom: 2px solid transparent;
}

.tab:hover {
  background: #f0f0f0;
}

.tab.active {
  color: #007bff;
  border-bottom-color: #007bff;
  background: #fff;
}

.tab-content {
  padding: 20px;
}

.footer {
  padding: 15px 20px;
  border-top: 1px solid #e0e0e0;
  text-align: center;
  font-size: 13px;
  color: #666;
}

.footer p {
  margin: 0;
}

kbd {
  display: inline-block;
  padding: 3px 8px;
  font-size: 12px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  box-shadow: inset 0 -1px 0 #ddd;
}
</style>
