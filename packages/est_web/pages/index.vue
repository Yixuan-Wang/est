<script setup lang="tsx">
const searchBox = ref<string>('')

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
    <h1 class="text-[5em] mb-4 lg:mb-8 transform font-bold">Est</h1>
    <ClientOnly>
      <SearchInput
        v-model="searchBox"
        class="w-full max-w-2xl"
        @search="go"
      />
    </ClientOnly>
    <div class="flex-[2]" />
  </main>
</template>
