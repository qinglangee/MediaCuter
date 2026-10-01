<template>
  <div class="time-input" :class="{ focused }">
    <button class="step-btn" @click="step(-100)" title="后退 100ms">&lt;</button>
    <input
      ref="inputRef"
      :value="displayValue"
      @focus="onFocus"
      @blur="onBlur"
      @input="onInput"
      @keydown="onKeydown"
      maxlength="12"
    />
    <button class="step-btn" @click="step(100)" title="前进 100ms">&gt;</button>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { formatTime, parseTime } from '../utils/time.js';

const props = defineProps({
  modelValue: { type: Number, default: 0 },
  max: { type: Number, default: 0 },
  label: { type: String, default: '' },
});

const emit = defineEmits(['update:modelValue']);

const inputRef = ref(null);
const focused = ref(false);
const editText = ref('');

const displayValue = computed(() =>
  focused.value ? editText.value : formatTime(props.modelValue)
);

function onFocus() {
  focused.value = true;
  editText.value = formatTime(props.modelValue);
  requestAnimationFrame(() => inputRef.value?.select());
}

function onBlur() {
  focused.value = false;
  const parsed = parseTime(editText.value);
  if (parsed !== null) {
    emit('update:modelValue', Math.min(parsed, props.max));
  }
}

function onInput(e) {
  editText.value = e.target.value;
}

function onKeydown(e) {
  if (e.key === 'Enter') {
    e.target.blur();
  } else if (e.key === 'Escape') {
    editText.value = formatTime(props.modelValue);
    e.target.blur();
  }
}

function step(delta) {
  const newVal = Math.max(0, Math.min(props.max, props.modelValue + delta));
  emit('update:modelValue', newVal);
}
</script>

<style scoped>
.time-input {
  display: flex;
  align-items: center;
  gap: 4px;
}

.time-input input {
  width: 130px;
  padding: 4px 8px;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 14px;
  background: #2a2a2a;
  color: #e0e0e0;
  border: 1px solid #555;
  border-radius: 4px;
  text-align: center;
}

.time-input input:focus {
  border-color: #409eff;
  outline: none;
}

.step-btn {
  padding: 4px 8px;
  background: #333;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.step-btn:hover {
  background: #444;
}
</style>
