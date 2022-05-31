<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getCareerDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="error.exists">
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <AlertMessage class="mt-2" v-bind:message="error.message"/>
        </div>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴</h3>
          <!--
          <div v-if="identity !== null" data-test="identity-set" class="m-4 text-2xl grid grid-cols-3">
            <div class="mt-2 justify-self-start col-span-1">名前</div><div class="justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
            <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
            <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
            <div class="mt-2 justify-self-start col-span-3">住所</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="justify-self-start col-span-2">{{ identity.prefecture }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="justify-self-start col-span-2">{{ identity.city }}</div>
            <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="justify-self-start col-span-2">{{ identity.address_line1 }}</div>
            <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="justify-self-start col-span-2">{{ identity.address_line2 }}</div>
            <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="justify-self-start col-span-2">{{ identity.telephone_number }}</div>
          </div>
          <p v-else data-test="no-identity-set" class="m-4 text-xl">職務経歴を取得出来ませんでした。</p>
          <button data-test="move-to-identity-page-button" v-on:click="moveToIdentityPage" class="mt-4 bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">ユーザー情報を編集する</button>
          -->
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TheHeader from '@/components/TheHeader.vue'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { useGetCareer } from '@/util/personalized/career-detail/useGetCareer'

export default defineComponent({
  name: 'CareerDetailPage',
  components: {
    TheHeader
  },
  setup () {
    const error = reactive({
      exists: false,
      message: ''
    })
    const careerId = ref('' as string)
    const route = useRoute()
    const router = useRouter()
    const {
      getCareerDone,
      getCareerFunc
    } = useGetCareer()
    onMounted(async () => {
      try {
        const resp = await refresh()
        if (resp instanceof RefreshResp) {
          // セッションが存在し、利用規約に同意済のため、ログイン後のページを表示可能
          // TODO: 正常系の処理
          // TODO: 実装メモ
          careerId.value = route.params.career_id as string
        } else if (resp instanceof ApiErrorResp) {
          const code = resp.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
          } else if (code === Code.NOT_TERMS_OF_USE_AGREED_YET) {
            await router.push('/terms-of-use')
          }
          // TODO: エラー処理
        }
      } catch (e) {
        // TODO: エラー処理
      }
    })
    return { error, getCareerDone, careerId }
  }
})
</script>
