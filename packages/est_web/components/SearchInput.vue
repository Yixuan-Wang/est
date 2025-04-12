<!-- eslint-disable @typescript-eslint/unified-signatures -->
<script setup lang="tsx">
interface SearchInputEmits {
  (e: 'search'): void;
  (e: 'autocomplete'): void;
}

const modelValue = defineModel<string>();
const emit = defineEmits<SearchInputEmits>();

const inputEl = ref<HTMLDivElement | null>(null);
const overlayEl = ref<HTMLDivElement | null>(null);

// Handle input changes for contenteditable div
function handleInput(_e: Event) {
  if (inputEl.value) {
    modelValue.value = inputEl.value.textContent || '';
    syncScrollPosition();
  }
}

watch(modelValue, (newValue) => {
  if (inputEl.value && newValue && newValue !== inputEl.value.textContent) {
    inputEl.value.textContent = newValue;

    nextTick(() => {
      const range = document.createRange();
      const sel = window.getSelection();
      range.selectNodeContents(inputEl.value!);
      range.collapse(false);
      sel?.removeAllRanges();
      sel?.addRange(range);
    });
  }
});

// Synchronize scroll position between input and overlay
function syncScrollPosition() {
  if (inputEl.value && overlayEl.value) {
    overlayEl.value.scrollLeft = inputEl.value.scrollLeft;
  }
}

// Split the input into segments for highlighting
const segments = computed(() => {
  if (!modelValue.value) return [
    { text: '@py', highlighted: 'text-gray-400' },
    { text: ' ', highlighted: '' },
    { text: 'print', highlighted: 'text-gray-300' },
  ]
  
  // Split the input by spaces, but keep all spaces inside `parts`
  const parts = modelValue.value.split(/(\s+)/)

  return parts.map((part) => {
    let highlighted;
    if (part.startsWith('@')) {
      highlighted = 'text-sky-500'
    } else if (part.startsWith('!')) {
      highlighted = 'text-blue-500'
    } else {
      highlighted = ''
    }
    return { text: part, highlighted }
  })
});

// Handle focus and cursor position
function focusInput() {
  inputEl.value?.focus();
}

// Handle key events for contenteditable div
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault();
    emit('search');
  } else if (e.key === 'Escape') {
    e.preventDefault();
    modelValue.value = '';
    if (inputEl.value) {
      inputEl.value.textContent = '';
    }
  } else if (e.key === 'Tab') {
    e.preventDefault();
    emit('autocomplete');
  }
}

// Expose focus method to parent
defineExpose({
  focus: focusInput
});
</script>

<template>
  <div
    class="relative w-full"
    role="search"
    @click="focusInput"
  >
    <!-- Contenteditable div replacing input -->
    <div
      ref="inputEl"
      contenteditable="true"
      spellcheck="false"
      autocomplete="off"
      autofocus="true"
      class="w-full px-4 py-2 z-1 text-lg relative whitespace-nowrap overflow-x-auto overflow-y-hidden"
      @input="handleInput"
      @scroll="syncScrollPosition"
      @keydown="handleKeydown"
    />
    
    <!-- Syntax highlighting overlay -->
    <div 
      ref="overlayEl"
      class="absolute z-10 top-0 left-0 px-4 py-2 text-lg pointer-events-none w-full overflow-hidden whitespace-pre"
      aria-hidden="true"
    >
      <template v-for="(segment, index) in segments" :key="index">
        <span 
          :class="segment.highlighted"
        >{{ segment.text }}</span>
      </template>
    </div>
  </div>
</template>

<style scoped>
[contenteditable] {
  color: transparent;
  caret-color: black;
  outline: none !important;
}
</style>