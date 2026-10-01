<template>
  <div class="editor audio-editor" @dragover.prevent @drop.prevent="onDrop">
    <div v-if="!filePath" class="drop-zone" @click="selectFile">
      <p>点击选择音频文件 或 拖拽文件到此处</p>
    </div>

    <template v-else>
      <div class="file-info">
        <span class="file-name">{{ fileName }}</span>
        <button class="close-btn" @click="clearFile">x</button>
      </div>

      <div class="waveform-section">
        <div class="waveform-scroll" ref="waveformScrollRef">
          <canvas
            ref="canvasRef"
            :width="canvasWidth"
            :height="160"
            @mousedown="onWaveformMouseDown"
          ></canvas>
        </div>
      </div>

      <div class="section">
        <div class="section-label">剪辑范围</div>
        <RangeSlider
          :duration="duration"
          :start="clipStart"
          :end="clipEnd"
          :playbackPosition="currentTime"
          :containerWidth="containerWidth"
          @update:start="setClipStart"
          @update:end="setClipEnd"
          @seek="seekTo"
          @update:scrollLeft="onRangeScroll"
        />
      </div>

      <div class="controls-row">
        <button @click="togglePlay">{{ isPlaying ? '暂停' : '播放' }}</button>
        <span class="time-display">{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
      </div>

      <div class="time-row">
        <div class="time-field">
          <label>开始:</label>
          <TimeInput :modelValue="clipStart" :max="clipEnd" @update:modelValue="setClipStart" />
        </div>
        <div class="time-field">
          <label>结束:</label>
          <TimeInput :modelValue="clipEnd" :max="duration" @update:modelValue="setClipEnd" />
        </div>
      </div>

      <div class="output-section">
        <div class="output-row">
          <label>输出目录:</label>
          <input v-model="outputDir" class="output-input" />
          <button @click="selectOutputDir">浏览</button>
        </div>
        <div class="output-row">
          <label>输出文件:</label>
          <input v-model="outputFilename" class="output-input" />
        </div>
        <div class="command-section">
          <label>FFmpeg 命令:</label>
          <div class="command-display">{{ commandStr }}</div>
          <button class="execute-btn" @click="execute" :disabled="isExecuting">
            {{ isExecuting ? '执行中...' : '执行剪辑' }}
          </button>
        </div>
        <div v-if="progress >= 0" class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
          <span class="progress-text">{{ formatTime(progressTime) }}</span>
        </div>
      </div>
    </template>

    <audio ref="audioRef" @timeupdate="onAudioTimeUpdate" @ended="onAudioEnded"></audio>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { formatTime } from '../utils/time.js';
import { generateCommand, buildFfmpegArgs } from '../utils/ffmpeg.js';
import RangeSlider from './RangeSlider.vue';
import TimeInput from './TimeInput.vue';

const canvasRef = ref(null);
const waveformScrollRef = ref(null);
const audioRef = ref(null);
const filePath = ref('');
const fileName = ref('');
const duration = ref(0);
const currentTime = ref(0);
const isPlaying = ref(false);
const clipStart = ref(0);
const clipEnd = ref(0);
const outputDir = ref('');
const outputFilename = ref('');
const commandStr = ref('');
const isExecuting = ref(false);
const progress = ref(-1);
const progressTime = ref(0);
const containerWidth = ref(800);
const peaks = ref([]);
const canvasWidth = ref(800);
let animFrameId = null;
let unlistenDrop = null;
let unlistenProgress = null;

const fullOutputPath = computed(() => {
  if (!outputDir.value || !outputFilename.value) return '';
  const sep = outputDir.value.endsWith('\\') || outputDir.value.endsWith('/') ? '' : '\\';
  return outputDir.value + sep + outputFilename.value;
});

function updateCommand() {
  if (!filePath.value || !fullOutputPath.value) {
    commandStr.value = '';
    return;
  }
  commandStr.value = generateCommand(filePath.value, clipStart.value, clipEnd.value, fullOutputPath.value);
}

function setClipStart(v) {
  clipStart.value = Math.min(v, clipEnd.value);
  updateCommand();
}

function setClipEnd(v) {
  clipEnd.value = Math.max(v, clipStart.value);
  updateCommand();
}

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a', 'opus'] }],
  });
  if (selected && typeof selected === 'string') {
    await loadFile(selected);
  }
}

async function loadFile(path) {
  filePath.value = path;
  const parts = path.replace(/\\/g, '/').split('/');
  fileName.value = parts[parts.length - 1];
  outputDir.value = parts.slice(0, -1).join('/').replace(/\//g, '\\');

  try {
    const meta = await invoke('get_video_metadata', { filePath: path });
    duration.value = meta.duration_ms;
    clipStart.value = 0;
    clipEnd.value = meta.duration_ms;

    const uniquePath = await invoke('get_unique_output_path', {
      dir: outputDir.value,
      filename: generateOutputName(fileName.value),
    });
    const uParts = uniquePath.replace(/\\/g, '/').split('/');
    outputFilename.value = uParts[uParts.length - 1];

    if (audioRef.value) {
      audioRef.value.src = `file:///${path.replace(/\\/g, '/')}`;
    }

    const waveformData = await invoke('get_waveform_data', { filePath: path });
    peaks.value = waveformData.peaks;
    canvasWidth.value = containerWidth.value;
    await nextTick();
    drawWaveform();
    updateCommand();
  } catch (e) {
    console.error('Failed to load audio:', e);
  }
}

function generateOutputName(originalName) {
  const dotIdx = originalName.lastIndexOf('.');
  const stem = dotIdx > 0 ? originalName.substring(0, dotIdx) : originalName;
  const ext = dotIdx > 0 ? originalName.substring(dotIdx + 1) : '';
  return ext ? `${stem}_clip.${ext}` : `${stem}_clip`;
}

function clearFile() {
  filePath.value = '';
  fileName.value = '';
  duration.value = 0;
  currentTime.value = 0;
  clipStart.value = 0;
  clipEnd.value = 0;
  peaks.value = [];
  outputDir.value = '';
  outputFilename.value = '';
  commandStr.value = '';
  if (audioRef.value) audioRef.value.src = '';
  if (canvasRef.value) {
    const ctx = canvasRef.value.getContext('2d');
    ctx.clearRect(0, 0, canvasRef.value.width, canvasRef.value.height);
  }
}

function drawWaveform() {
  const canvas = canvasRef.value;
  if (!canvas || peaks.value.length === 0) return;
  const ctx = canvas.getContext('2d');
  const w = canvas.width;
  const h = canvas.height;
  const mid = h / 2;

  ctx.fillStyle = '#1a1a1a';
  ctx.fillRect(0, 0, w, h);

  ctx.strokeStyle = '#409eff';
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let i = 0; i < w && i < peaks.value.length; i++) {
    const amplitude = peaks.value[i] * mid * 0.9;
    ctx.moveTo(i + 0.5, mid - amplitude);
    ctx.lineTo(i + 0.5, mid + amplitude);
  }
  ctx.stroke();

  ctx.strokeStyle = 'rgba(64, 158, 255, 0.15)';
  ctx.fillStyle = 'rgba(64, 158, 255, 0.1)';
  const startX = duration.value > 0 ? (clipStart.value / duration.value) * w : 0;
  const endX = duration.value > 0 ? (clipEnd.value / duration.value) * w : 0;
  ctx.fillRect(startX, 0, endX - startX, h);
  ctx.strokeStyle = '#409eff';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(startX, 0);
  ctx.lineTo(startX, h);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(endX, 0);
  ctx.lineTo(endX, h);
  ctx.stroke();
}

function drawPlayhead() {
  drawWaveform();
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  const w = canvas.width;
  const h = canvas.height;

  if (duration.value > 0) {
    const x = (currentTime.value / duration.value) * w;
    ctx.strokeStyle = '#f56c6c';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, h);
    ctx.stroke();
  }
}

function onAudioTimeUpdate() {
  if (audioRef.value) {
    currentTime.value = Math.round(audioRef.value.currentTime * 1000);
    drawPlayhead();
  }
}

function onAudioEnded() {
  isPlaying.value = false;
}

function togglePlay() {
  if (!audioRef.value) return;
  if (isPlaying.value) {
    audioRef.value.pause();
    isPlaying.value = false;
  } else {
    audioRef.value.play();
    isPlaying.value = true;
  }
}

function seekTo(ms) {
  if (audioRef.value) {
    audioRef.value.currentTime = ms / 1000;
    currentTime.value = ms;
    drawPlayhead();
  }
}

function onWaveformMouseDown(e) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const ms = (x / canvas.width) * duration.value;
  seekTo(Math.max(0, Math.min(duration.value, ms)));

  const onMove = (ev) => {
    const mx = ev.clientX - rect.left;
    const mms = (mx / canvas.width) * duration.value;
    seekTo(Math.max(0, Math.min(duration.value, mms)));
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

function onRangeScroll(scrollLeft) {
  // keep waveform and range slider visually in sync if needed
}

async function selectOutputDir() {
  const selected = await open({ directory: true });
  if (selected && typeof selected === 'string') {
    outputDir.value = selected;
    updateCommand();
  }
}

async function execute() {
  if (!commandStr.value || isExecuting.value) return;
  isExecuting.value = true;
  progress.value = 0;
  progressTime.value = 0;

  const args = buildFfmpegArgs(filePath.value, clipStart.value, clipEnd.value, fullOutputPath.value);

  try {
    const result = await invoke('execute_ffmpeg', { commandStr: args, outputPath: fullOutputPath.value });
    progress.value = 100;
    alert('剪辑完成！输出文件: ' + result);
  } catch (e) {
    alert('剪辑失败: ' + e);
  } finally {
    isExecuting.value = false;
    setTimeout(() => { progress.value = -1; }, 2000);
  }
}

function onDrop(e) {
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.path) {
      loadFile(file.path);
    }
  }
}

function updateContainerWidth() {
  containerWidth.value = Math.max(400, window.innerWidth - 60);
  canvasWidth.value = containerWidth.value;
  if (peaks.value.length > 0) {
    nextTick(() => drawWaveform());
  }
}

watch([clipStart, clipEnd], () => {
  if (peaks.value.length > 0) drawWaveform();
});

onMounted(async () => {
  updateContainerWidth();
  window.addEventListener('resize', updateContainerWidth);

  unlistenDrop = await listen('file-dropped', (event) => {
    if (event.payload) {
      loadFile(event.payload);
    }
  });

  unlistenProgress = await listen('ffmpeg-progress', (event) => {
    if (event.payload.time_ms !== undefined) {
      progressTime.value = event.payload.time_ms;
      const clipDuration = clipEnd.value - clipStart.value;
      if (clipDuration > 0) {
        progress.value = Math.min(99, (event.payload.time_ms / clipDuration) * 100);
      }
    }
    if (event.payload.completed) {
      progress.value = 100;
    }
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', updateContainerWidth);
  unlistenDrop?.();
  unlistenProgress?.();
  if (animFrameId) cancelAnimationFrame(animFrameId);
});
</script>

<style scoped>
.audio-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px;
}

.drop-zone {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed #555;
  border-radius: 8px;
  cursor: pointer;
  color: #888;
  font-size: 16px;
  min-height: 200px;
}

.drop-zone:hover {
  border-color: #409eff;
  color: #409eff;
}

.file-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: #2a2a2a;
  border-radius: 4px;
}

.file-name {
  font-size: 13px;
  color: #ccc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 16px;
  padding: 0 4px;
}

.close-btn:hover {
  color: #f56c6c;
}

.waveform-section {
  background: #1a1a1a;
  border-radius: 4px;
  overflow: hidden;
}

.waveform-scroll {
  overflow-x: auto;
}

.waveform-scroll canvas {
  display: block;
  cursor: pointer;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.section-label {
  font-size: 12px;
  color: #888;
}

.controls-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.controls-row button {
  padding: 6px 16px;
  background: #409eff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.controls-row button:hover {
  background: #66b1ff;
}

.time-display {
  font-family: 'Consolas', monospace;
  font-size: 14px;
  color: #ccc;
}

.time-row {
  display: flex;
  gap: 20px;
  align-items: center;
}

.time-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.time-field label {
  font-size: 13px;
  color: #aaa;
}

.output-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #333;
}

.output-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.output-row label {
  font-size: 13px;
  color: #aaa;
  white-space: nowrap;
}

.output-input {
  flex: 1;
  padding: 4px 8px;
  background: #2a2a2a;
  color: #e0e0e0;
  border: 1px solid #555;
  border-radius: 4px;
  font-size: 13px;
}

.output-row button {
  padding: 4px 12px;
  background: #333;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}

.output-row button:hover {
  background: #444;
}

.command-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.command-section label {
  font-size: 13px;
  color: #aaa;
}

.command-display {
  padding: 8px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 4px;
  font-family: 'Consolas', monospace;
  font-size: 12px;
  color: #4fc3f7;
  word-break: break-all;
  white-space: pre-wrap;
}

.execute-btn {
  padding: 8px 20px;
  background: #67c23a;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  align-self: flex-start;
}

.execute-btn:hover {
  background: #85ce61;
}

.execute-btn:disabled {
  background: #555;
  cursor: not-allowed;
}

.progress-bar {
  position: relative;
  height: 20px;
  background: #2a2a2a;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #67c23a;
  transition: width 0.3s;
}

.progress-text {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 11px;
  color: #ccc;
  font-family: 'Consolas', monospace;
}

audio {
  display: none;
}
</style>
