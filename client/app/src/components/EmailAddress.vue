<template>
  <div class="mb-6 pt-3 rounded bg-gray-200">
    <label class="block text-gray-700 text-sm font-bold mb-2 ml-3" for="email">メールアドレス</label>
    <input type="email" required minlength="1" maxlength="254" id="email" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'EmailAddress'
})
</script>
