<template>
  <div
    class="range-slider"
    @wheel.ctrl.prevent="onWheel"
    @wheel.meta.prevent="onWheel"
  >
    <div
      class="slider-scroll"
      ref="scrollRef"
      @scroll="onScroll"
    >
      <div class="slider-track" :style="{ width: trackWidth + 'px' }" @mousedown="onTrackClick">
        <div class="selected-region" :style="selectedRegionStyle"></div>
        <div
          class="handle handle-start"
          :style="{ left: startPos + 'px' }"
          @mousedown.prevent.stop="onStartDrag"
        ></div>
        <div
          class="handle handle-end"
          :style="{ left: endPos + 'px' }"
          @mousedown.prevent.stop="onEndDrag"
        ></div>
        <div
          v-if="playbackPosition >= 0"
          class="playhead"
          :style="{ left: playbackPos + 'px' }"
        ></div>
      </div>
    </div>
    <div class="zoom-controls">
      <button @click="zoomIn" title="放大">+</button>
      <span class="zoom-label">{{ zoom.toFixed(1) }}x</span>
      <button @click="zoomOut" title="缩小">-</button>
      <button @click="resetZoom" title="重置">1:1</button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';

const props = defineProps({
  duration: { type: Number, default: 0 },
  start: { type: Number, default: 0 },
  end: { type: Number, default: 0 },
  playbackPosition: { type: Number, default: -1 },
  containerWidth: { type: Number, default: 800 },
});

const emit = defineEmits(['update:start', 'update:end', 'seek', 'update:scrollLeft']);

const zoom = ref(1);
const scrollLeft = ref(0);
const scrollRef = ref(null);
const maxZoom = ref(50);

const trackWidth = computed(() => props.containerWidth * zoom.value);

const startPos = computed(() =>
  props.duration > 0 ? (props.start / props.duration) * trackWidth.value : 0
);

const endPos = computed(() =>
  props.duration > 0 ? (props.end / props.duration) * trackWidth.value : 0
);

const playbackPos = computed(() =>
  props.duration > 0 ? (props.playbackPosition / props.duration) * trackWidth.value : 0
);

const selectedRegionStyle = computed(() => ({
  left: startPos.value + 'px',
  width: Math.max(0, endPos.value - startPos.value) + 'px',
}));

function pxToMs(px) {
  return props.duration > 0 ? (px / trackWidth.value) * props.duration : 0;
}

function onScroll() {
  if (scrollRef.value) {
    scrollLeft.value = scrollRef.value.scrollLeft;
    emit('update:scrollLeft', scrollLeft.value);
  }
}

function onWheel(e) {
  const factor = e.deltaY < 0 ? 1.2 : 1 / 1.2;
  const newZoom = Math.max(1, Math.min(maxZoom.value, zoom.value * factor));

  const centerMs = ((scrollLeft.value + props.containerWidth / 2) / trackWidth.value) * props.duration;
  scrollLeft.value = Math.max(0,
    (centerMs / props.duration) * (props.containerWidth * newZoom) - props.containerWidth / 2
  );

  zoom.value = newZoom;
  requestAnimationFrame(() => {
    if (scrollRef.value) scrollRef.value.scrollLeft = scrollLeft.value;
  });
}

function zoomIn() {
  zoom.value = Math.min(maxZoom.value, zoom.value * 1.5);
}

function zoomOut() {
  zoom.value = Math.max(1, zoom.value / 1.5);
}

function resetZoom() {
  zoom.value = 1;
  scrollLeft.value = 0;
  requestAnimationFrame(() => {
    if (scrollRef.value) scrollRef.value.scrollLeft = 0;
  });
}

function onStartDrag(e) {
  const onMove = (ev) => {
    const rect = scrollRef.value.getBoundingClientRect();
    const x = ev.clientX - rect.left + scrollRef.value.scrollLeft;
    const ms = Math.max(0, Math.min(props.end, pxToMs(x)));
    emit('update:start', Math.round(ms));
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

function onEndDrag(e) {
  const onMove = (ev) => {
    const rect = scrollRef.value.getBoundingClientRect();
    const x = ev.clientX - rect.left + scrollRef.value.scrollLeft;
    const ms = Math.max(props.start, Math.min(props.duration, pxToMs(x)));
    emit('update:end', Math.round(ms));
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
}

function onTrackClick(e) {
  if (e.target.classList.contains('handle')) return;
  const rect = scrollRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left + scrollRef.value.scrollLeft;
  const ms = pxToMs(x);
  if (ms >= props.start && ms <= props.end) {
    emit('seek', Math.round(ms));
  }
}

onMounted(() => {
  maxZoom.value = Math.max(10, props.duration / 200);
});
</script>

<style scoped>
.range-slider {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.slider-scroll {
  overflow-x: auto;
  overflow-y: hidden;
  height: 40px;
  position: relative;
}

.slider-track {
  position: relative;
  height: 100%;
  background: #2a2a2a;
  border-radius: 4px;
  cursor: pointer;
}

.selected-region {
  position: absolute;
  top: 0;
  height: 100%;
  background: rgba(64, 158, 255, 0.25);
  border-left: 2px solid #409eff;
  border-right: 2px solid #409eff;
  pointer-events: none;
}

.handle {
  position: absolute;
  top: 0;
  width: 10px;
  height: 100%;
  background: #409eff;
  border-radius: 3px;
  cursor: ew-resize;
  transform: translateX(-50%);
  z-index: 2;
  transition: background 0.15s;
}

.handle:hover {
  background: #66b1ff;
}

.playhead {
  position: absolute;
  top: 0;
  width: 2px;
  height: 100%;
  background: #f56c6c;
  transform: translateX(-50%);
  pointer-events: none;
  z-index: 3;
}

.zoom-controls {
  display: flex;
  align-items: center;
  gap: 6px;
}

.zoom-controls button {
  padding: 2px 8px;
  font-size: 12px;
  background: #333;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 3px;
  cursor: pointer;
}

.zoom-controls button:hover {
  background: #444;
}

.zoom-label {
  font-size: 12px;
  color: #888;
  min-width: 40px;
  text-align: center;
}
</style>
