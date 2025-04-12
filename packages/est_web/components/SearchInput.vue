<script setup lang="tsx">
interface SearchInputProps {
  modelValue: string;
}

interface SearchInputEmits {
  (e: 'update:modelValue', value: string): void;
  (e: 'search'): void;
}

const props = defineProps<SearchInputProps>();
const emit = defineEmits<SearchInputEmits>();

const inputEl = ref<HTMLDivElement | null>(null);
const overlayEl = ref<HTMLDivElement | null>(null);

// Handle input changes for contenteditable div
function handleInput(_e: Event) {
  if (inputEl.value) {
    emit('update:modelValue', inputEl.value.textContent || '');
    syncScrollPosition();
  }
}

// Synchronize scroll position between input and overlay
function syncScrollPosition() {
  if (inputEl.value && overlayEl.value) {
    overlayEl.value.scrollLeft = inputEl.value.scrollLeft;
  }
}

// Split the input into segments for highlighting
const segments = computed(() => {
  if (!props.modelValue) return [
    { text: '@py', highlighted: 'text-gray-400' },
    { text: ' ', highlighted: '' },
    { text: 'print', highlighted: 'text-gray-300' },
  ]
  
  // Split the input by spaces, but keep all spaces inside `parts`
  const parts = props.modelValue.split(/(\s+)/).filter(part => part.length > 0)

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
    emit('update:modelValue', '');
    if (inputEl.value) {
      inputEl.value.textContent = '';
    }
  }
}

// Expose focus method to parent
defineExpose({
  focus: focusInput
});
</script>

<template>
  <div
    class="relative w-full rounded-lg shadow-md"
    role="search"
    @click="focusInput"
  >
    <!-- Contenteditable div replacing input -->
    <div
      ref="inputEl"
      contenteditable="true"
      spellcheck="false"
      autocomplete="off"
      class="w-full px-4 py-2 text-lg bg-white bg-opacity-80 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent relative whitespace-nowrap overflow-x-auto overflow-y-hidden"
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
}
</style>