<template>
  <div class="flex justify-center">
    <!-- ref: https://github.com/tailwindlabs/tailwindcss/discussions/2945#discussioncomment-143252 -->
    <svg class="animate-spin h-16 w-16 rounded-full bg-transparent border-4 border-transparent border-opacity-50 border-r-white border-t-white" viewBox="0 0 24 24"></svg>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'WaitingCircle'
})
</script>
