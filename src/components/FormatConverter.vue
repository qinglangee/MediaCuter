<template>
  <div class="converter" @dragover.prevent @drop.prevent="onDrop">
    <div v-if="!filePath" class="drop-zone" @click="selectFile">
      <p>点击选择视频或音频文件 或 拖拽文件到此处</p>
    </div>

    <template v-else>
      <div class="file-info">
        <span class="file-name">{{ fileName }}</span>
        <span class="file-meta">{{ fileMeta }}</span>
        <button class="close-btn" @click="clearFile">x</button>
      </div>

      <div class="section">
        <div class="section-label">输出格式:</div>
        <select v-model="outputExt" class="format-select">
          <option v-for="fmt in formats" :key="fmt.ext" :value="fmt.ext">{{ fmt.label }}</option>
        </select>
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
          <button class="execute-btn" @click="execute" :disabled="isExecuting || !commandStr">
            {{ isExecuting ? '执行中...' : '执行转换' }}
          </button>
        </div>
        <div v-if="progress >= 0" class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
          <span class="progress-text">{{ progressText }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { getOutputFormats, generateConvertCommand, buildConvertArgs } from '../utils/ffmpeg.js';

const filePath = ref('');
const fileName = ref('');
const fileMeta = ref('');
const hasVideo = ref(false);
const duration = ref(0);
const outputExt = ref('');
const formats = ref([]);
const outputDir = ref('');
const outputFilename = ref('');
const commandStr = ref('');
const isExecuting = ref(false);
const progress = ref(-1);
const progressText = ref('');
let unlistenDrop = null;
let unlistenProgress = null;

const fullOutputPath = computed(() => {
  if (!outputDir.value || !outputFilename.value) return '';
  const sep = outputDir.value.endsWith('\\') || outputDir.value.endsWith('/') ? '' : '\\';
  return outputDir.value + sep + outputFilename.value;
});

function updateCommand() {
  if (!filePath.value || !outputExt.value || !fullOutputPath.value) {
    commandStr.value = '';
    return;
  }
  commandStr.value = generateConvertCommand(filePath.value, outputExt.value, fullOutputPath.value);
}

watch(outputExt, () => {
  updateOutputFilename();
  updateCommand();
});

watch(fullOutputPath, () => {
  updateCommand();
});

function updateOutputFilename() {
  if (!fileName.value || !outputExt.value) return;
  const dotIdx = fileName.value.lastIndexOf('.');
  const stem = dotIdx > 0 ? fileName.value.substring(0, dotIdx) : fileName.value;
  outputFilename.value = `${stem}.${outputExt.value}`;
}

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [
      { name: '媒体文件', extensions: ['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'ts', 'mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a', 'wma', 'opus'] },
    ],
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
    hasVideo.value = meta.has_video;
    duration.value = meta.duration_ms;

    const durSec = Math.round(duration.value / 1000);
    const min = Math.floor(durSec / 60);
    const sec = durSec % 60;
    const typeStr = meta.has_video ? '视频' : '音频';
    const streams = meta.streams.map(s => s.codec_name || s.codec_type).join(', ');
    fileMeta.value = `${typeStr} | ${min}:${sec.toString().padStart(2, '0')} | ${streams}`;

    formats.value = getOutputFormats(meta.has_video);
    if (formats.value.length > 0 && !formats.value.find(f => f.ext === outputExt.value)) {
      outputExt.value = formats.value[0].ext;
    }

    await invoke('allow_asset_path', { path });

    const uniquePath = await invoke('get_unique_output_path', {
      dir: outputDir.value,
      filename: outputFilename.value || generateDefaultOutputName(fileName.value),
    });
    const uParts = uniquePath.replace(/\\/g, '/').split('/');
    outputFilename.value = uParts[uParts.length - 1];

    updateCommand();
  } catch (e) {
    console.error('Failed to load file:', e);
  }
}

function generateDefaultOutputName(originalName) {
  const dotIdx = originalName.lastIndexOf('.');
  const stem = dotIdx > 0 ? originalName.substring(0, dotIdx) : originalName;
  const ext = outputExt.value || 'mp4';
  return `${stem}_converted.${ext}`;
}

function clearFile() {
  filePath.value = '';
  fileName.value = '';
  fileMeta.value = '';
  hasVideo.value = false;
  duration.value = 0;
  outputExt.value = '';
  formats.value = [];
  outputDir.value = '';
  outputFilename.value = '';
  commandStr.value = '';
  progress.value = -1;
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
  progressText.value = '00:00:00';

  const args = buildConvertArgs(filePath.value, outputExt.value, fullOutputPath.value);

  try {
    const result = await invoke('execute_ffmpeg', { commandStr: args, outputPath: fullOutputPath.value });
    progress.value = 100;
    progressText.value = '完成';
    alert('转换完成！输出文件: ' + result);
  } catch (e) {
    alert('转换失败: ' + e);
  } finally {
    isExecuting.value = false;
    setTimeout(() => { progress.value = -1; }, 3000);
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

onMounted(async () => {
  unlistenDrop = await listen('file-dropped', (event) => {
    if (event.payload) {
      loadFile(event.payload);
    }
  });

  unlistenProgress = await listen('ffmpeg-progress', (event) => {
    if (event.payload.time_ms !== undefined) {
      const ms = event.payload.time_ms;
      const h = Math.floor(ms / 3600000);
      const m = Math.floor((ms % 3600000) / 60000);
      const s = Math.floor((ms % 60000) / 1000);
      progressText.value = `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
      if (duration.value > 0) {
        progress.value = Math.min(99, (ms / duration.value) * 100);
      }
    }
    if (event.payload.completed) {
      progress.value = 100;
      progressText.value = '完成';
    }
  });

  return () => {
    unlistenDrop?.();
    unlistenProgress?.();
  };
});

onBeforeUnmount(() => {
  unlistenDrop?.();
  unlistenProgress?.();
});
</script>

<style scoped>
.converter {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
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
  gap: 12px;
  padding: 8px 12px;
  background: #2a2a2a;
  border-radius: 4px;
}

.file-name {
  font-size: 14px;
  color: #e0e0e0;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  font-size: 12px;
  color: #888;
  white-space: nowrap;
}

.close-btn {
  margin-left: auto;
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

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  font-size: 13px;
  color: #aaa;
}

.format-select {
  padding: 8px 12px;
  background: #2a2a2a;
  color: #e0e0e0;
  border: 1px solid #555;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  max-width: 300px;
}

.format-select:focus {
  outline: none;
  border-color: #409eff;
}

.output-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 8px;
  padding-top: 12px;
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
  min-width: 65px;
}

.output-input {
  flex: 1;
  padding: 6px 10px;
  background: #2a2a2a;
  color: #e0e0e0;
  border: 1px solid #555;
  border-radius: 4px;
  font-size: 13px;
}

.output-row button {
  padding: 6px 14px;
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
  padding: 10px;
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
  padding: 8px 24px;
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
  height: 22px;
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
</style>
