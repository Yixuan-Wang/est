<script setup lang="tsx">

const searchBox = ref<string>('')
const searchBoxEl = ref<HTMLDivElement | null>(null)
const dropdownEl = ref<HTMLDivElement | null>(null)
const dropdownContainerEl = ref<HTMLDivElement | null>(null)
const engines = ref<string[]>([])

const suggestion = ref<{
  text: string;
  replace: (_: string) => string;
}[]>([]);

if (import.meta.client) {
  $fetch<{ engines: string[] }>(`/api/experimental/engines`)
    .then((engineList) => {
      try {
        engines.value = engineList.engines.filter(engine => engine && !engine.endsWith("_")).sort((a, b) => a.length - b.length) ?? []
      }
      catch (e) {
        console.error('Error fetching engines:', e)
      }
    })
    .catch((error) => {
      console.error('Error fetching engines:', error)
    })
}



const { height: heightSearchBox } = useElementBounding(searchBoxEl);
const { height: heightDropdown } = useElementBounding(dropdownEl);
const heightDropdownContainer = computed(() => {
  return heightSearchBox.value + heightDropdown.value
})

watch(searchBox, (newValue) => {
  if (newValue.endsWith('@')) {
    suggestion.value = engines.value.slice(0, 10).map(engine => ({
      text: `@${engine}`,
      replace: (input) => input.replace(/@\S*$/, `@${engine}`),
    }))
    return
  }

  if (newValue.match(/@\S*$/)) {
    const engine = newValue.match(/@\S*$/)![0].slice(1)
    const filteredEngines = engines.value.filter(e => e.startsWith(engine))
    suggestion.value = filteredEngines.slice(0, 10).map(e => ({
      text: `@${e}`,
      replace: (input) => input.replace(/@\S*$/, `@${e}`),
    }))
    return
  }
  
  suggestion.value = [];
})

function acceptSuggestion(suggestionItem: { text: string; replace: (_: string) => string }) {
  searchBox.value = suggestionItem.replace(searchBox.value);
}

function autocomplete() {
  if (suggestion.value.length) acceptSuggestion(suggestion.value[0]);
}

function go() {
  // Navigate to the search URL with the current searchBox value as a query parameter
  if (searchBox.value.trim()) {
    const encodedQuery = encodeURIComponent(searchBox.value.trim());
    const baseURL = useRuntimeConfig().app.baseURL;
    window.open(`${baseURL}search?q=${encodedQuery}`, '_blank');
  }
}
</script>

<template>
  <main class="px-4 flex flex-col items-center justify-between w-full h-screen bg-gray-100 relative">
    <div class="flex-1" />
    <h1 class="text-[5em] lg:mb-4 transform font-bold">Est</h1>
    <ClientOnly>
      <div
        class="relative w-full"
      >
        <SearchInput
          ref="searchBoxEl"
          v-model="searchBox"
          class="w-full z-1"
          @search="go"
          @autocomplete="autocomplete"
        />
        <div
          ref="dropdownEl"
          class="absolute top-full left-0 right-0 z-1"
        >
          <div
            v-show="suggestion.length"
            class="px-4 grid grid-cols-1 gap-1"
            :class="{
              'pb-4': suggestion.length
            }"
          >
            <TransitionGroup
              name="suggestion"
              tag="div"
            >
              <div 
                v-for="(s, i) in suggestion" 
                :key="i"
                @click="acceptSuggestion(s)"
              >
                {{ s.text }}
              </div>
            </TransitionGroup>
          </div>
        </div>
        <div
          ref="dropdownContainerEl"
          class="absolute top-0 left-0 right-0 h-full rounded-lg bg-white border border-gray-300 shadow-md z-0 transition-height duration-150 ease-out"
          :style="{
            height: heightDropdownContainer ? `${heightDropdownContainer}px` : '100%',
          }"
        />
      </div>
    </ClientOnly>
    <div class="flex-[2]" />
  </main>
</template>

<style scoped>
main > * {
  max-width: 65ch;
}

.suggestion-enter-active {
  transition: opacity 0.2s ease-out 0.15s;
}

.suggestion-leave-active {
  transition: opacity 0.15s ease-in;
}

.suggestion-enter-from,
.suggestion-leave-to {
  opacity: 0;
}
</style>