#!/usr/bin/env python3
"""
VoicePicker TTS Service
使用 Qwen3-TTS-0.6B 模型进行文本转语音
"""

import argparse
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
except ImportError as e:
    print(f"Error: 缺少依赖 - {e}", file=sys.stderr)
    print("请运行：pip install -r requirements.txt", file=sys.stderr)
    sys.exit(1)


# 默认参考音频路径
DEFAULT_REF_AUDIO = os.path.join(
    os.path.dirname(os.path.dirname(__file__)),
    "resources",
    "reference_audio.wav"
)

# 默认参考文本（与参考音频内容匹配）
DEFAULT_REF_TEXT = "以后电信诈骗，估计可以复制别人的声音，进行诈骗了。"


class QwenTtsEngine:
    """Qwen3-TTS-0.6B 引擎"""

    def __init__(self, model_path: str = None, ref_audio_path: str = None):
        """
        初始化 TTS 引擎

        Args:
            model_path: 模型路径，默认为 models/qwen3-tts-0.6b
            ref_audio_path: 参考音频路径，用于语音克隆
        """
        if model_path is None:
            model_path = os.path.join(
                os.path.dirname(os.path.dirname(__file__)),
                "models",
                "qwen3-tts-0.6b"
            )

        self.model_path = model_path
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.model = None
        self._loaded = False
        self._ref_audio_path = ref_audio_path or DEFAULT_REF_AUDIO
        self._ref_text = DEFAULT_REF_TEXT

    def load_model(self):
        """加载模型"""
        if self._loaded:
            return

        print(f"正在加载模型从：{self.model_path}", file=sys.stderr)
        print(f"使用设备：{self.device}", file=sys.stderr)

        try:
            # 使用 qwen-tts 库加载模型
            self.model = Qwen3TTSModel.from_pretrained(
                self.model_path,
                device_map=self.device if self.device == "cuda" else None,
                dtype=torch.float32,  # macOS 不支持 bfloat16，使用 float32
            )
            self._loaded = True
            print(f"模型加载完成", file=sys.stderr)

            # 检查参考音频是否存在
            if not os.path.exists(self._ref_audio_path):
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
                    self._ref_audio_path = fallback_audio
                    print(f"使用备用参考音频：{self._ref_audio_path}", file=sys.stderr)
                else:
                    print(f"警告：参考音频不存在：{self._ref_audio_path}", file=sys.stderr)
            else:
                print(f"使用参考音频：{self._ref_audio_path}", file=sys.stderr)

        except Exception as e:
            print(f"模型加载失败：{e}", file=sys.stderr)
            raise

    def synthesize(
        self,
        text: str,
        speed: float = 1.0,
        volume: float = 1.0,
        language: str = "Chinese"
    ) -> bytes:
        """
        合成语音

        Args:
            text: 输入文本
            speed: 语速 (0.5-2.0) - 注：当前模型暂不支持语速控制
            volume: 音量 (0.0-1.0)
            language: 语言（默认：Chinese）

        Returns:
            WAV 格式的音频字节
        """
        if not self._loaded:
            self.load_model()

        # 检查参考音频
        if not os.path.exists(self._ref_audio_path):
            raise FileNotFoundError(
                f"参考音频不存在：{self._ref_audio_path}\n"
                "请提供一个参考音频文件用于语音克隆。"
            )

        # 使用 generate_voice_clone 方法
        with torch.no_grad():
            wavs, sr = self.model.generate_voice_clone(
                text=text,
                language=language,
                ref_audio=self._ref_audio_path,
                ref_text=self._ref_text,
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


def main():
    parser = argparse.ArgumentParser(description='VoicePicker TTS Service')
    parser.add_argument('--text', '-t', required=True, help='要转换的文本')
    parser.add_argument('--speed', '-s', type=float, default=1.0, help='语速 (0.5-2.0)')
    parser.add_argument('--volume', '-v', type=float, default=1.0, help='音量 (0.0-1.0)')
    parser.add_argument('--output', '-o', default='-', help='输出文件路径，- 表示 stdout')
    parser.add_argument('--model', '-m', default=None, help='模型路径')
    parser.add_argument('--ref-audio', '-r', default=None, help='参考音频路径')
    parser.add_argument('--ref-text', default=None, help='参考音频文本')
    parser.add_argument('--language', default='Chinese', help='语言')

    args = parser.parse_args()

    # 创建引擎
    engine = QwenTtsEngine(model_path=args.model, ref_audio_path=args.ref_audio)

    if args.ref_text:
        engine._ref_text = args.ref_text

    try:
        # 合成语音
        audio_data = engine.synthesize(
            args.text,
            args.speed,
            args.volume,
            language=args.language
        )

        # 输出
        if args.output == '-':
            # 写入 stdout（二进制模式）
            sys.stdout.buffer.write(audio_data)
            sys.stdout.buffer.flush()
        else:
            # 写入文件
            with open(args.output, 'wb') as f:
                f.write(audio_data)
            print(f"音频已保存到：{args.output}", file=sys.stderr)

    except Exception as e:
        print(f"TTS 合成失败：{e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
