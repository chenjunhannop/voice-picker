#!/bin/bash
# 测试后端 Tauri 命令

echo "=== 测试 Tauri 命令 ==="

# 使用 curl 调用前端来测试 invoke
cd /Users/chenjunhan/VoicePicker

# 测试 1: 检查端口 1420 上的前端是否运行
echo "测试 1: 检查前端"
curl -s http://localhost:1420/ > /dev/null && echo "前端正在运行" || echo "前端未运行"

# 测试 2: 检查 TTS 服务
echo ""
echo "测试 2: 手动启动 TTS 服务"
pkill -f "tts_service" 2>/dev/null
/Users/chenjunhan/VoicePicker/python/.venv/bin/python3 /Users/chenjunhan/VoicePicker/python/tts_service.py --preload --host 127.0.0.1 --port 8765 &
TTS_PID=$!
echo "TTS 进程 ID: $TTS_PID"

echo "等待服务启动..."
sleep 10

# 测试 3: 健康检查
echo ""
echo "测试 3: 健康检查"
curl -s http://127.0.0.1:8765/health

# 测试 4: 合成测试
echo ""
echo ""
echo "测试 4: 合成测试"
curl -s -X POST http://127.0.0.1:8765/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "你好测试", "speed": 1.0, "volume": 1.0}' | head -c 100

# 清理
kill $TTS_PID 2>/dev/null

echo ""
echo "=== 测试完成 ==="
