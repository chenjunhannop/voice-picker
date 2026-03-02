#!/bin/bash
# VoicePicker 端到端测试脚本

echo "=== VoicePicker E2E 测试 ==="
echo ""

# 1. 检查应用是否运行
echo "1. 检查应用进程"
if pgrep -f "voice-picker" > /dev/null; then
    echo "   ✓ 应用正在运行"
else
    echo "   ✗ 应用未运行"
    exit 1
fi

# 2. 检查 TTS 服务
echo ""
echo "2. 检查 TTS HTTP 服务"
health_response=$(curl -s http://127.0.0.1:8765/health 2>/dev/null)
if [ -n "$health_response" ]; then
    echo "   ✓ TTS 服务响应：$health_response"
else
    echo "   ✗ TTS 服务无响应"
fi

# 3. 测试合成 API
echo ""
echo "3. 测试 TTS 合成 API"
synth_response=$(curl -s -X POST http://127.0.0.1:8765/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "测试语音", "speed": 1.0, "volume": 1.0}' 2>/dev/null)
if [ -n "$synth_response" ]; then
    echo "   ✓ 合成响应：${synth_response:0:100}..."
else
    echo "   ✗ 合成无响应"
fi

# 4. 查看应用日志
echo ""
echo "4. 查看应用日志"
tail -50 /private/tmp/claude-501/-Users-chenjunhan/tasks/b308e47.output 2>/dev/null | grep -E "(快捷键|TTS|设置|播放|错误)" || echo "   没有找到相关日志"

echo ""
echo "=== 测试完成 ==="
