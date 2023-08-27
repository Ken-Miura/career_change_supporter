<template>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <header class="max-w-lg mx-auto">
      <router-link to="/">
        <h1 class="text-xl lg:text-2xl font-bold text-white text-center">就職先・転職先を見極めるためのサイト</h1>
      </router-link>
    </header>
    <main class="bg-white max-w-3xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
      <h1 class="font-bold text-2xl">OSSライセンス表記</h1>
      <p class="mt-2">このWebサイトが含めているOSSに関して、そのライセンスに従い、必要な記載事項を下記に列挙します。</p>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/indutny/fft.js/">FFT.js</a></h2>
        <p class="ml-4">Copyright Fedor Indutny, 2017.</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/indutny/fft.js/#license">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/skyway/js-sdk">SkyWay JS-SDK</a></h2>
        <p class="ml-4">Copyright (c) 2023 WebRTC Platform SkyWay</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/skyway/js-sdk/blob/main/LICENSE">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/tailwindlabs/tailwindcss">tailwindcss</a></h2>
        <p class="ml-4">Copyright (c) Tailwind Labs, Inc.</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/tailwindlabs/tailwindcss/blob/master/LICENSE">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/vuejs/core">Vue.js</a></h2>
        <p class="ml-4">Copyright (c) 2018-present, Yuxi (Evan) You</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/vuejs/core/blob/main/LICENSE">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/vuejs/router">Vue Router</a></h2>
        <p class="ml-4">Copyright (c) 2019-present Eduardo San Martin Morote</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/vuejs/router/blob/main/LICENSE">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/vuejs/vuex">Vuex</a></h2>
        <p class="ml-4">Copyright (c) 2015-present Evan You</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/vuejs/vuex/blob/main/LICENSE">Released under the MIT license</a></p>
      </div>
      <div class="mt-4">
        <h2 class="font-bold"><a class="hover:underline" href="https://github.com/tserkov/vue-plugin-load-script">vue-plugin-load-script</a></h2>
        <p class="ml-4">Copyright (c) 2017 James Churchard</p>
        <p class="ml-4"><a class="hover:underline" href="https://github.com/tserkov/vue-plugin-load-script/blob/master/LICENSE.md">Released under the MIT license</a></p>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'OssLicensePage'
})
</script>
