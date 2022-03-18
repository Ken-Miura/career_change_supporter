<template>
  <div class="pt-3 rounded bg-gray-200">
    <label class="block text-gray-700 text-sm font-bold mb-2 ml-3">メールアドレス</label>
    <input v-on:input="onInput" type="email" required minlength="1" maxlength="254" class="bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'EmailAddressInput',
  setup (_, { emit }) {
    const onInput = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof HTMLInputElement)) {
        // HTMLInputElement以外が来るときはinputタグ以外に関数が指定されている。
        // inputタグ以外にしていすることは想定していないため、Errorとする。
        throw new Error(`!(target instanceof HTMLInputElement): target is ${target}`)
      }
      emit('on-email-address-updated', target.value)
    }
    return {
      onInput
    }
  }
})
</script>
