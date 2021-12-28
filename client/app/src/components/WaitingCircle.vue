<template>
  <div class="flex justify-center">
    <!-- TODO: tailwind css 3.0にアップグレード後、styleの項目をclassへと移動する (vueのtailwind cssプラグインが、まだtailwind css 3.0に対応していないため、それが対応後、アップグレード) -->
    <!-- ref: https://github.com/tailwindlabs/tailwindcss/discussions/2945#discussioncomment-143252 -->
    <svg class="animate-spin h-16 w-16 rounded-full bg-transparent border-2 border-transparent border-opacity-50" style="border-right-color: white; border-top-color: white;" viewBox="0 0 24 24"></svg>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'WaitingCircle'
})
</script>
