# VoicePicker

macOS 本地 TTS 划词朗读软件 - 使用 Qwen3-TTS-0.6B 模型提供高质量语音合成。

## 功能特性

- 🔥 **全局快捷键** - 选中文字后按 `Cmd+Option+X` 即可朗读
- 🎯 **本地 TTS** - 使用阿里通义 Qwen3-TTS-0.6B 模型，无需联网
- 🎵 **播放控制** - 支持播放/暂停/停止
- 📝 **历史记录** - 自动保存朗读历史
- ⚙️ **自定义设置** - 可调节语速、音量、快捷键

## 系统要求

- macOS 12.0+
- 2GB 可用内存（模型加载需要）
- 2GB 可用磁盘空间（模型文件）

## 安装

### 方法一：从 DMG 安装（推荐）

1. 从 [Releases](https://github.com/yourusername/VoicePicker/releases) 下载最新版的 `.dmg` 文件
2. 双击打开 DMG
3. 将 VoicePicker 拖到 Applications 文件夹

### 方法二：源码构建

#### 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node.js (v18+)
brew install node

# 安装 Python 3.12
brew install python@3.12
```

#### 安装依赖

```bash
cd VoicePicker

# 安装 Node.js 依赖
npm install

# 安装 Python 依赖
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
cd ..

# 下载 TTS 模型
mkdir -p models
cd models
# 使用 huggingface-cli 下载（需要先安装：pip install huggingface_hub）
huggingface-cli download Qwen/Qwen3-TTS-12Hz-0.6B-Base --local-dir qwen3-tts-0.6b
cd ..
```

#### 构建应用

```bash
# 开发模式运行
npm run tauri dev

# 生产构建
npm run tauri build
```

构建完成后，应用位于：
- `src-tauri/target/release/bundle/macos/VoicePicker.app`
- `src-tauri/target/release/bundle/dmg/VoicePicker_*.dmg`

## 使用说明

### 基本使用

1. 启动 VoicePicker，应用会显示在菜单栏
2. 在任何应用中选中要朗读的文字
3. 按下快捷键 `Cmd+Option+X`
4. 应用会自动复制选中的文字并朗读

### 播放控制

- **播放** - 点击播放按钮重新播放当前音频
- **暂停** - 点击暂停按钮暂停播放
- **停止** - 点击停止按钮停止播放

### 设置

- **快捷键** - 自定义触发朗读的快捷键
- **语速** - 调节朗读速度（0.5x - 2.0x）
- **音量** - 调节音量大小（0% - 100%）

## 项目结构

```
VoicePicker/
├── src/                        # 前端源码 (Vue 3)
│   ├── components/
│   │   ├── SettingsPanel.vue   # 设置面板
│   │   ├── PlayerControls.vue  # 播放控制
│   │   ├── HistoryList.vue     # 历史记录
│   │   └── StatusIndicator.vue # 状态指示器
│   ├── App.vue
│   ├── main.ts
│   └── style.css
├── src-tauri/                  # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs             # 应用入口
│   │   ├── hotkey.rs           # 快捷键处理
│   │   ├── clipboard.rs        # 剪贴板监控
│   │   ├── tts/
│   │   │   ├── mod.rs
│   │   │   └── engine.rs       # TTS 引擎接口
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   └── player.rs       # 音频播放器
│   │   └── config/
│   │       ├── mod.rs
│   │       └── settings.rs     # 设置管理
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── entitlements.plist
├── python/                     # Python TTS 服务
│   ├── tts_server.py           # TTS 服务主进程
│   └── requirements.txt
├── models/                     # TTS 模型文件
│   └── qwen3-tts-0.6b/
└── package.json
```

## 技术栈

| 层次 | 技术 |
|------|------|
| 应用框架 | Tauri v2 |
| TTS 引擎 | Qwen3-TTS-0.6B |
| UI 框架 | Vue 3 + TypeScript |
| 音频播放 | rodio |
| 全局快捷键 | tauri-plugin-global-shortcut |

## 常见问题

### Q: 首次启动很慢？
A: 首次启动时需要加载 TTS 模型到内存（约 1.5GB），这是正常现象。后续启动会快很多。

### Q: 朗读没有声音？
A: 检查系统音量设置，确保输出设备正常。也可以尝试重新插拔耳机。

### Q: 快捷键不生效？
A: 可能与其他应用快捷键冲突。请在设置中更改快捷键，或关闭冲突的应用。

### Q: 选中的文字为空？
A: 确保正确选中了文字再按快捷键。某些应用（如 PDF 阅读器）可能不支持标准复制操作。

### Q: 模型文件在哪里下载？
A: 模型托管在 HuggingFace: https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base

## 开发计划

- [ ] 支持更多 TTS 引擎（macOS 内置、Edge TTS 等）
- [ ] 多语言支持
- [ ] 语音克隆功能（3 秒样本）
- [ ] 导出音频文件
- [ ] 批处理模式（朗读长文档）

## License

Apache 2.0 License

## 鸣谢

- [Qwen3-TTS](https://github.com/QwenLM/Qwen3-TTS) - 阿里通义开源 TTS 模型
- [Tauri](https://tauri.app/) - 轻量级跨平台应用框架
- [Vue 3](https://vuejs.org/) - 渐进式 JavaScript 框架
