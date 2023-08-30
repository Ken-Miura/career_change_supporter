<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getTempMfaSecretDone && postEnableMfaReqDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="errMessageOnOpen" class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="errMessageOnOpen"/>
      </div>
      <div v-else class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 data-test="description" class="font-bold text-xl">下記の手順を実施して二段階認証を有効化して下さい。</h3>
        <ol class="mt-4 ml-6 list-decimal font-bold text-base lg:text-xl">
          <li data-test="qr-code-label" class="mt-2">認証アプリを起動し、QRコードを読み込んで下さい。</li>
          <div class="flex justify-center w-full">
            <img data-test="qr-code-value" class="mt-2" v-bind:src="base64EncodedImageUrl" />
          </div>
          <p data-test="secret-label" class="mt-2">QRコードが読み込めない場合、次の文字列をキーとして認証アプリに手動で入力して下さい。</p>
          <p data-test="secret-value" class="mt-2 ml-1 lg:ml-2 text-sm lg:text-xl">{{ base32EncodedSecret }}</p>
          <li data-test="pass-code-label" class="mt-4">認証アプリに表示された数値を入力して、下記の送信を押して下さい。</li>
          <form @submit.prevent="submitPassCodeToEnableMfa">
            <PassCodeInput @on-pass-code-updated="setPassCode"/>
            <button data-test="submit-button" type="submit" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">送信</button>
          </form>
        </ol>
        <div v-if="errMessageOnSubmit">
          <AlertMessage class="mt-4 ml-6" v-bind:message="errMessageOnSubmit"/>
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
import { usePostEnableMfaReq } from '@/util/personalized/enable-mfa-confirmation/usePostEnableMfaReq'
import { PostEnableMfaReqResp } from '@/util/personalized/enable-mfa-confirmation/PostEnableMfaReqResp'
import { useStore } from 'vuex'
import { SET_RECOVERY_CODE } from '@/store/mutationTypes'
import { usePassCode } from '@/components/usePassCode'
import PassCodeInput from '@/components/PassCodeInput.vue'

export default defineComponent({
  name: 'EnableMfaConfirmationPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle,
    PassCodeInput
  },
  setup () {
    const router = useRouter()
    const store = useStore()
    const base64EncodedImage = ref('')
    const base64EncodedImageUrl = computed(() => {
      return `data:image/png;base64,${base64EncodedImage.value}`
    })
    const base32EncodedSecret = ref('')

    const errMessageOnOpen = ref(null as string | null)
    const errMessageOnSubmit = ref(null as string | null)

    const {
      getTempMfaSecretDone,
      getTempMfaSecretFunc
    } = useGetTempMfaSecret()

    const {
      postEnableMfaReqDone,
      postEnableMfaReqFunc
    } = usePostEnableMfaReq()

    const {
      passCode,
      setPassCode
    } = usePassCode()

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
          errMessageOnOpen.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const tmpMfaSecret = resp.getTempMfaSecret()
        base64EncodedImage.value = tmpMfaSecret.base64_encoded_image
        base32EncodedSecret.value = tmpMfaSecret.base32_encoded_secret
      } catch (e) {
        errMessageOnOpen.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const submitPassCodeToEnableMfa = async () => {
      try {
        const resp = await postEnableMfaReqFunc(passCode.value)
        if (!(resp instanceof PostEnableMfaReqResp)) {
          if (!(resp instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${resp}`)
          }
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            // 二段階認証の設定を変更する画面からログイン画面へ急に遷移するとユーザーを混乱させるのでメッセージ表示にする
            errMessageOnSubmit.value = Message.UNAUTHORIZED_ON_MFA_SETTING_OPERATION_MESSAGE
            return
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            // 二段階認証の設定を変更する画面から利用規約画面へ急に遷移するとユーザーを混乱させるのでメッセージ表示にする
            errMessageOnSubmit.value = Message.NOT_TERMS_OF_USE_AGREED_YET_ON_MFA_SETTING_OPERATION_MESSAGE
            return
          }
          errMessageOnSubmit.value = createErrorMessage(resp.getApiError().getCode())
          return
        }
        const recoveryCode = resp.getRecoveryCode()
        store.commit(SET_RECOVERY_CODE, recoveryCode)
        await router.push('/enable-mfa-success')
      } catch (e) {
        errMessageOnSubmit.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    return {
      getTempMfaSecretDone,
      base64EncodedImageUrl,
      base32EncodedSecret,
      passCode,
      submitPassCodeToEnableMfa,
      errMessageOnOpen,
      errMessageOnSubmit,
      postEnableMfaReqDone,
      setPassCode
    }
  }
})
</script>
