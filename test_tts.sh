#!/bin/bash
# VoicePicker TTS 测试脚本

echo "=== VoicePicker TTS 测试 ==="
echo ""

# 测试 1: 检查 TTS 服务是否运行
echo "测试 1: 检查 TTS 服务状态"
response=$(curl -s http://127.0.0.1:8765/health 2>/dev/null)
if [ -n "$response" ]; then
    echo "  TTS 服务响应：$response"
else
    echo "  TTS 服务未运行"
fi
echo ""

# 测试 2: 发送合成请求
echo "测试 2: 发送 TTS 合成请求"
response=$(curl -s -X POST http://127.0.0.1:8765/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "你好，这是测试语音", "speed": 1.0, "volume": 1.0}' 2>/dev/null)
echo "  响应：$response"
echo ""

# 测试 3: 检查 Rust 应用日志
echo "测试 3: 查看 Rust 应用日志"
tail -30 /private/tmp/claude-501/-Users-chenjunhan/tasks/b51c9c3.output 2>/dev/null | grep -E "(VoicePicker|TTS)"
echo ""

echo "=== 测试完成 ==="
