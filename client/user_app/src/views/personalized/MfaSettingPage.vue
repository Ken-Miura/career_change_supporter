<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!postTempMfaSecretDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-2xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 data-test="mfa-setting-label" class="font-bold text-2xl">二段回認証設定</h3>
        <p data-test="mfa-setting-description" class="mt-4 ml-2 text-xl">二段回認証の設定を変更します。本サービスにおける二段階認証には認証アプリを利用します。二段階認証を有効化するためには、事前にスマートフォンにGoogle Authenticator (<a class="hover:underline" href="https://apps.apple.com/jp/app/google-authenticator/id388497605">iOS版リンク</a>、<a class="hover:underline" href="https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2&hl=ja&gl=US">Android OS版リンク</a>) またはそれに準ずる認証アプリをインストールして下さい。</p>
        <div class="mt-2 ml-4 grid grid-cols-3">
          <p data-test="mfa-enabled-label" class="mt-4 justify-self-start text-xl col-span-2">現在の二段回認証の設定</p>
          <p data-test="mfa-enabled-value" class="mt-4 justify-self-center text-xl col-span-1">{{ mfaStatus }}</p>
        </div>
        <button data-test="change-mfa-setting-button" v-on:click="changeMfaSetting" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">{{ mfaBtnLabel }}</button>
        <div v-if="errMessage">
          <AlertMessage class="mt-6" v-bind:message="errMessage"/>
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
import { useRoute, useRouter } from 'vue-router'
import { toBoolean } from '@/util/ToBoolean'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code, createErrorMessage } from '@/util/Error'
import { Message } from '@/util/Message'
import { usePostTempMfaSecret } from '@/util/personalized/mfa-setting/usePostTempMfaSecret'
import { PostTempMfaSecretResp } from '@/util/personalized/mfa-setting/PostTempMfaSecretResp'

export default defineComponent({
  name: 'MfaSettingPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const route = useRoute()
    const query = route.query
    const queryResult = toBoolean(query['mfa-enabled'] as string)
    const mfaEnabled = ref(queryResult)
    const mfaStatus = computed(() => {
      if (mfaEnabled.value) {
        return '有効'
      } else {
        return '無効'
      }
    })
    const mfaBtnLabel = computed(() => {
      if (mfaEnabled.value) {
        return '無効化する'
      } else {
        return '有効化する'
      }
    })

    const {
      postTempMfaSecretDone,
      postTempMfaSecretFunc
    } = usePostTempMfaSecret()

    onMounted(async () => {
      try {
        const resp = await refresh()
        if (!(resp instanceof RefreshResp)) {
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
        }
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    })

    const enableMfa = async () => {
      try {
        const resp = await postTempMfaSecretFunc()
        if (!(resp instanceof PostTempMfaSecretResp)) {
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
        await router.push('/enable-mfa-confirmation')
      } catch (e) {
        errMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const changeMfaSetting = async () => {
      if (mfaEnabled.value) {
        console.log('TODO: 無効化')
      } else {
        await enableMfa()
      }
    }

    const errMessage = ref(null as string | null)

    return {
      postTempMfaSecretDone,
      mfaStatus,
      mfaBtnLabel,
      changeMfaSetting,
      errMessage
    }
  }
})
</script>
