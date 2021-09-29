<template>
  <div class="mb-6 pt-3 rounded bg-gray-200">
    <label class="block text-gray-700 text-sm font-bold mb-2 ml-3" for="password">{{ label }}</label>
    <input type="password" required pattern="[!-~]{10,32}" title="英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列" id="password" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'Password',
  props: {
    label: String
  }
})
</script>
