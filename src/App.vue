<template>
  <div class="app">
    <div class="tabs">
      <button :class="{ active: activeTab === 'video' }" @click="activeTab = 'video'">视频剪辑</button>
      <button :class="{ active: activeTab === 'audio' }" @click="activeTab = 'audio'">音频剪辑</button>
    </div>
    <div class="content">
      <VideoEditor v-if="activeTab === 'video'" />
      <AudioEditor v-if="activeTab === 'audio'" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import VideoEditor from './components/VideoEditor.vue';
import AudioEditor from './components/AudioEditor.vue';

const activeTab = ref('video');
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', sans-serif;
  background: #1e1e1e;
  color: #e0e0e0;
  overflow: hidden;
}

.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.tabs {
  display: flex;
  background: #252525;
  border-bottom: 1px solid #333;
  padding: 0 10px;
}

.tabs button {
  padding: 10px 24px;
  background: transparent;
  color: #888;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.tabs button:hover {
  color: #ccc;
}

.tabs button.active {
  color: #409eff;
  border-bottom-color: #409eff;
}

.content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: #1e1e1e;
}

::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #666;
}
</style>
