<template>
  <div class="bg-red-100 border border-red-400 text-red-700 pt-3 px-4 py-3 rounded relative" role="alert">
    <strong class="font-bold">Holy smokes!</strong>
    <span class="block sm:inline">{{message}}</span>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'AlertMessage',
  props: {
    message: String
  }
})
</script>
