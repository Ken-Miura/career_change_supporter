<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main>
      <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 data-test="label" class="font-bold text-xl text-center">二段階認証を有効化しました。</h3>
        <p data-test="description" class="mt-4 text-lg text-center">認証アプリを含んだ端末を紛失した際に利用するリカバリーコードを下記に記載します。端末の紛失に備えて<span class=" text-red-500">下記のリカバリーコードをコピー&ペーストし、安全な場所に保管して下さい。</span></p>
        <p data-test="recovery-code" v-if="recoveryCode" class="mt-4 font-bold text-xl text-center">{{ recoveryCode }}</p>
        <div v-else>
          <p data-test="no-recovery-code-found-label" class="mt-4 font-bold text-xl text-center">リカバリーコードを表示できません</p>
          <p data-test="no-recovery-code-found-value" class="mt-2 text-lg text-center">リカバリーコードは一度しか表示されません。リカバリーコードをコピー&ペーストして保管していない場合、二段階認証を無効化し、再度有効化する手順を実施して下さい。</p>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/profile" class="hover:underline text-center">プロフィールへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'EnableMfaSuccessPage',
  components: {
    TheHeader
  },
  setup () {
    const store = useStore()
    const recoveryCode = ref(null as string | null)

    onMounted(async () => {
      recoveryCode.value = store.state.recoveryCode
    })

    return {
      recoveryCode
    }
  }
})
</script>
