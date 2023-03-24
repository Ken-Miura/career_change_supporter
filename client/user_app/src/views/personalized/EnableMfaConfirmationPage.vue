<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getTempMfaSecretDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-xl">下記の手順を実施して二段階認証を有効化して下さい。</h3>
        <ol class="mt-4 ml-6 list-decimal font-bold text-xl">
          <li class="mt-2">認証アプリを起動し、QRコードを読み込んで下さい。</li>
          <div class="flex justify-center w-full">
            <img class="mt-2" v-bind:src="base64EncodedImageUrl" />
          </div>
          <p class="mt-2 text-lg">QRコードが読み込めない場合、次の文字列をキーとして手動で入力して下さい。</p>
          <p class="mt-2 ml-2 text-lg">{{ base32EncodedSecret }}</p>
          <li class="mt-4">認証アプリに表示された数値を入力して、下記の送信を押して下さい。</li>
          <form @submit.prevent="submitPassCodeToEnableMfa">
            <div class="mt-2 w-full justify-self-start col-span-6 pt-3 pl-2 rounded bg-gray-200">
              <input v-model="passCode" type="text" inputmode="numeric" pattern="[0-9]{6}" title="半角数字のみの6桁でご入力下さい。" required minlength="6" maxlength="6" class="text-right bg-gray-200 rounded w-full text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500 px-3 pb-3">
            </div>
            <button type="submit" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">送信</button>
          </form>
        </ol>
        <div v-if="errMessage">
          <AlertMessage class="mt-4 ml-6" v-bind:message="errMessage"/>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, onMounted, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRouter } from 'vue-router'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { useGetTempMfaSecret } from '@/util/personalized/enable-mfa-confirmation/useGetTempMfaSecret'
import { GetTempMfaSecretResp } from '@/util/personalized/enable-mfa-confirmation/GetTempMfaSecretResp'

export default defineComponent({
  name: 'EnableMfaConfirmationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const base64EncodedImage = ref('')
    const base64EncodedImageUrl = computed(() => {
      return `data:image/png;base64,${base64EncodedImage.value}`
    })
    const base32EncodedSecret = ref('')
    const passCode = ref('')

    const errMessage = ref(null as string | null)

    const {
      getTempMfaSecretDone,
      getTempMfaSecretFunc
    } = useGetTempMfaSecret()

    onMounted(async () => {
      try {
        const resp = await getTempMfaSecretFunc()
        if (!(resp instanceof GetTempMfaSecretResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
            return
          }
          errMessage.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const tmpMfaSecret = resp.getTempMfaSecret()
        base64EncodedImage.value = tmpMfaSecret.base64_encoded_image
        base32EncodedSecret.value = tmpMfaSecret.base32_encoded_secret
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const submitPassCodeToEnableMfa = async () => {
      console.log('submitPassCodeToEnableMfa')
    }

    return {
      getTempMfaSecretDone,
      base64EncodedImageUrl,
      base32EncodedSecret,
      passCode,
      submitPassCodeToEnableMfa,
      errMessage
    }
  }
})
</script>
