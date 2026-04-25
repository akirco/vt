# vt - visual terminal(开发中...)

**用 Rust 打造的终端媒体播放器，让你在命令行里看视频！**

## 项目简介

vt 是一个使用 Rust 编写的终端媒体播放器，支持通过 **Sixel** 和 **Kitty Graphics Protocol** 在终端中渲染视频。项目灵感来源于 [buddy](https://github.com/JVSCHANDRADITHYA/buddy) 项目，但使用 Rust 重构，带来更好的性能和内存安全性。

## 核心特性

- **视频播放** - 基于 FFmpeg 解码，支持常见视频格式
- **音频播放** - 可选音频输出（--audio 参数）
- **双协议支持** - 自动检测 Kitty/Sixel 终端图形协议
- **丰富色彩** - Sixel 模式支持 2-256 色和多种抖动算法
- **自由缩放** - 通过 --scale 调整视频尺寸
- **状态显示** - verbose 模式实时显示 FPS 和帧数

## 🎮 快速使用

```bash
# 基本播放
vt video.mp4

# 启用音频
vt video.mp4 --audio

# 缩放播放
vt video.mp4 -s 0.5

# Sixel 模式调优
vt video.mp4 -c 128 -d fs -q high
```

## 同类项目对比

| 项目                                               | 语言   | 特点                                   |
| -------------------------------------------------- | ------ | -------------------------------------- |
| **vt**                                             | Rust   | 双协议、音频支持、高性能               |
| [timg](https://github.com/hzeller/timg)            | C++    | 功能最全，支持摄像头、PDF、多线程解码  |
| [chafa](https://github.com/hpjansson/chafa)        | C      | 图像转换神器，支持丰富符号集和色彩空间 |
| [viu](https://github.com/atanunq/viu)              | Rust   | 轻量图像查看器，支持 iTerm/Kitty       |
| [buddy](https://github.com/JVSCHANDRADITHYA/buddy) | Python | 灵感来源，3种渲染模式，区域平均降采样  |

**timg** - 老牌终端媒体查看器，支持图片、视频、GIF、PDF 甚至摄像头，功能极其丰富。

**chafa** - 终端图形界的瑞士军刀，支持从电传打字机到现代终端的各种设备，符号集极其丰富。

**viu** - 简洁的 Rust 图像查看器，3.1k+ stars，支持 iTerm/Kitty 协议和 Unicode 半块字符。

**buddy** - Python 编写的终端视频播放器，使用 24 位真彩色和 Unicode 半块字符，采用区域平均降采样算法。

## 灵感来源

本项目受 [buddy](https://github.com/JVSCHANDRADITHYA/buddy) 启发。buddy 是"Block-based Unicode Direct-color Display Yield"的缩写，使用 Python + NumPy 实现，通过 Unicode 半块字符 `▀` 实现双倍垂直分辨率，配合 24 位真彩色输出高质量终端视频。

vt 在 buddy 的理念基础上：

- 使用 **Rust** 重写，带来零成本抽象和内存安全
- 添加 **Sixel/Kitty 协议**支持，输出更高质量的图形
- 集成 **音频播放**功能
- 利用 **FFmpeg** 进行硬件加速解码

## 目前存在的问题

1. **音视频不同步** - 音频和视频独立播放，缺少时间戳同步机制
2. **帧同步误差累积** - `sync_frame` 使用 `Instant::now()` 重设时间，会导致漂移
3. **无交互控制** - 不支持暂停、快进、快退、音量调节
4. **缺少循环播放** - 视频播放完即退出，无法循环
5. **终端变化无响应** - 播放时改变终端大小不会自适应
6. **Sixel 输出锁问题** - 编码时释放 stdout lock 可能导致输出交错
7. **音频重采样未完全生效** - audio.rs 中未正确使用重采样后的数据
8. **无 fallback 机制** - 终端不支持 Sixel/Kitty 时直接无法工作
9. **RGB buffer 容量计算可能不准确** - 基于 width*height*3 估算可能不符合实际
10. **无文件合法性检查** - 未检查输入文件是否存在或可读
11. **图像大小与终端大小缩放算法存在问题**
12. **图片文件解码**

---

**项目地址**：[vt - Terminal Media Player](https://github.com/akirco/vt)
**灵感项目**：[buddy](https://github.com/JVSCHANDRADITHYA/buddy)

---
