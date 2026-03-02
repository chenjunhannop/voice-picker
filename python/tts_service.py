#!/usr/bin/env python3
"""
VoicePicker TTS HTTP Service
使用 Qwen3-TTS-0.6B 模型进行文本转语音，以 HTTP 服务形式运行
"""

import sys
import os
from pathlib import Path

# 添加项目根目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import torch
    from qwen_tts import Qwen3TTSModel
    import soundfile as sf
    import numpy as np
    import io
    from flask import Flask, request, jsonify
    from flask_cors import CORS
except ImportError as e:
    print(f"Error: 缺少依赖 - {e}", file=sys.stderr)
    print("请运行：pip install flask flask-cors", file=sys.stderr)
    sys.exit(1)


# 默认参考音频路径
DEFAULT_REF_AUDIO = os.path.join(
    os.path.dirname(os.path.dirname(__file__)),
    "resources",
    "reference_audio.wav"
)

# 默认参考文本
DEFAULT_REF_TEXT = "以后电信诈骗，估计可以复制别人的声音，进行诈骗了。"

app = Flask(__name__)
CORS(app)

# 全局模型实例
model = None
model_loaded = False


def load_model(model_path: str = None):
    """加载模型到全局状态"""
    global model, model_loaded

    if model_loaded:
        return

    if model_path is None:
        model_path = os.path.join(
            os.path.dirname(os.path.dirname(__file__)),
            "models",
            "qwen3-tts-0.6b"
        )

    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"[TTS Service] 正在加载模型从：{model_path}", file=sys.stderr)
    print(f"[TTS Service] 使用设备：{device}", file=sys.stderr)

    model = Qwen3TTSModel.from_pretrained(
        model_path,
        device_map=device if device == "cuda" else None,
        dtype=torch.float32,
    )

    model_loaded = True
    print(f"[TTS Service] 模型加载完成", file=sys.stderr)


def synthesize(
    text: str,
    speed: float = 1.0,
    volume: float = 1.0,
    speaker: str = None,
    language: str = "Chinese"
) -> bytes:
    """
    合成语音

    Args:
        text: 输入文本
        speed: 语速 (暂不支持)
        volume: 音量 (0.0-1.0)
        speaker: 说话人名称（可选）
        language: 语言

    Returns:
        WAV 格式的音频字节
    """
    global model

    if not model_loaded:
        raise RuntimeError("模型未加载")

    # 获取支持的说话人列表
    supported_speakers = model.get_supported_speakers()

    # 选择说话人
    if speaker is None:
        speaker = supported_speakers[0] if supported_speakers else "Vivian"

    # 参考音频路径
    ref_audio_path = DEFAULT_REF_AUDIO
    if not os.path.exists(ref_audio_path):
        # 使用 gradio 测试音频作为备用
        fallback_audio = os.path.join(
            os.path.dirname(os.path.dirname(__file__)),
            "python",
            ".venv",
            "lib",
            "python3.12",
            "site-packages",
            "gradio",
            "media_assets",
            "audio",
            "recording1.wav"
        )
        if os.path.exists(fallback_audio):
            ref_audio_path = fallback_audio

    # 使用 generate_voice_clone 方法
    with torch.no_grad():
        wavs, sr = model.generate_voice_clone(
            text=text,
            language=language,
            ref_audio=ref_audio_path,
            ref_text=DEFAULT_REF_TEXT,
            non_streaming_mode=True
        )

    # 获取音频波形
    audio = wavs[0]

    # 调整音量
    if volume != 1.0:
        audio = audio * volume
        audio = np.clip(audio, -1.0, 1.0)

    # 转换为 WAV 格式
    buffer = io.BytesIO()
    sf.write(
        buffer,
        audio,
        samplerate=sr,
        format='WAV'
    )

    return buffer.getvalue()


@app.route('/health', methods=['GET'])
def health():
    """健康检查端点"""
    return jsonify({
        'status': 'healthy' if model_loaded else 'loading',
        'model_loaded': model_loaded
    })


@app.route('/synthesize', methods=['POST'])
def api_synthesize():
    """TTS 合成 API"""
    try:
        data = request.get_json()

        if not data or 'text' not in data:
            return jsonify({'error': '缺少 text 参数'}), 400

        text = data['text']
        speed = data.get('speed', 1.0)
        volume = data.get('volume', 1.0)
        language = data.get('language', 'Chinese')

        print(f"[TTS Service] 合成请求：text='{text[:50]}...', speed={speed}, volume={volume}", file=sys.stderr)

        audio_data = synthesize(text, speed, volume, language=language)

        # 返回 Base64 编码的音频
        import base64
        audio_base64 = base64.b64encode(audio_data).decode('utf-8')

        return jsonify({
            'success': True,
            'audio_data': audio_base64,
            'length': len(audio_data)
        })

    except Exception as e:
        print(f"[TTS Service] 合成失败：{e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return jsonify({'error': str(e)}), 500


@app.route('/load_model', methods=['POST'])
def api_load_model():
    """手动触发模型加载"""
    try:
        data = request.get_json() or {}
        model_path = data.get('model_path')

        load_model(model_path)

        return jsonify({
            'success': True,
            'message': '模型加载完成'
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 500


def main():
    import argparse

    parser = argparse.ArgumentParser(description='VoicePicker TTS HTTP Service')
    parser.add_argument('--host', default='127.0.0.1', help='监听地址')
    parser.add_argument('--port', type=int, default=8765, help='监听端口')
    parser.add_argument('--model', default=None, help='模型路径')
    parser.add_argument('--preload', action='store_true', help='启动时预加载模型')

    args = parser.parse_args()

    # 启动时加载模型
    if args.preload:
        load_model(args.model)

    # 启动 HTTP 服务
    print(f"[TTS Service] 启动 HTTP 服务：http://{args.host}:{args.port}", file=sys.stderr)
    app.run(host=args.host, port=args.port, threaded=True)


if __name__ == '__main__':
    main()
