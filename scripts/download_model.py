#!/usr/bin/env python3
"""
VoicePicker 模型下载脚本
下载 Qwen3-TTS-0.6B 模型到 models 目录
"""

import os
import sys
from pathlib import Path

try:
    from huggingface_hub import snapshot_download
except ImportError:
    print("错误：缺少 huggingface_hub 库")
    print("请运行：pip install huggingface_hub")
    sys.exit(1)


def download_model():
    """下载 Qwen3-TTS-0.6B 模型"""

    # 模型仓库
    repo_id = "Qwen/Qwen3-TTS-12Hz-0.6B-Base"

    # 本地保存路径
    script_dir = Path(__file__).parent
    models_dir = script_dir.parent / "models"
    model_path = models_dir / "qwen3-tts-0.6b"

    print(f"正在下载模型到：{model_path}")
    print(f"模型大小约 1.5GB，请耐心等待...")

    try:
        # 创建目录
        models_dir.mkdir(exist_ok=True)

        # 下载模型
        snapshot_download(
            repo_id=repo_id,
            local_dir=model_path,
            local_dir_use_symlinks=False,
            resume_download=True
        )

        print(f"\n模型下载完成！")
        print(f"模型路径：{model_path}")

        # 验证模型文件
        required_files = ["config.json", "model.safetensors", "tokenizer.json"]
        missing = []

        for file in required_files:
            if not (model_path / file).exists():
                missing.append(file)

        if missing:
            print(f"\n警告：缺少以下文件：{', '.join(missing)}")
            print("模型可能下载不完整，请重试。")
        else:
            print("\n模型文件验证通过 ✓")

    except Exception as e:
        print(f"\n下载失败：{e}")
        print("\n请检查网络连接，或尝试使用代理。")
        sys.exit(1)


if __name__ == "__main__":
    download_model()
