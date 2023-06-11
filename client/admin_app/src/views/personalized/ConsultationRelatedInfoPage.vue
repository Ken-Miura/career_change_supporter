<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!requestsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="outerErrorMessage" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="outerErrorMessage"/>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談</h3>
          <div v-if="!consultationErrMessage">
            <div v-if="consultation" class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">相談番号</div><div class="mt-2 justify-self-start col-span-2">{{ consultation.consultation_id }}</div>
              <div class="mt-2 justify-self-start col-span-1">コンサルタントID</div><div class="mt-2 justify-self-start col-span-2">{{ consultation.consultant_id }}</div>
              <div class="mt-2 justify-self-start col-span-1">ユーザーアカウントID</div><div class="mt-2 justify-self-start col-span-2">{{ consultation.user_account_id }}</div>
              <div class="mt-2 justify-self-start col-span-1">相談日時</div><div class="mt-2 justify-self-start col-span-2">{{ consultation.meeting_at }}</div>
              <div class="mt-2 justify-self-start col-span-1">部屋名</div><div class="mt-2 justify-self-start col-span-2">{{ consultation.room_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">ユーザー入室日時</div><div v-if="consultation.user_account_entered_at" class="mt-2 justify-self-start col-span-2">{{ consultation.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">入室記録なし</div>
              <div class="mt-2 justify-self-start col-span-1">コンサルタント入室日時</div><div v-if="consultation.consultant_entered_at" class="mt-2 justify-self-start col-span-2">{{ consultation.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">入室記録なし</div>
            </div>
            <div v-else class="m-4 text-2xl">
              相談は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">ユーザーとしての相談一覧</h3>
          <!-- <div v-if="!consultationsAsUserErrMessage">
            <div v-if="consultationsAsUser.length !== 0">
              <ul>
                <li v-for="consultationAsUser in consultationsAsUser" v-bind:key="consultationAsUser.consultation_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談番号{{ consultationAsUser.consultation_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-7">
                    <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.consultant_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.meeting_at }}</div>
                    <div class="mt-2 justify-self-start col-span-3">部屋名</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.room_name }}</div>
                    <div class="mt-2 justify-self-start col-span-3">ユーザー入室日時</div><div v-if="consultationAsUser.user_account_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 justify-self-start col-span-3">コンサルタント入室日時</div><div v-if="consultationAsUser.consultant_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 w-full justify-self-start col-span-7">
                      <button v-on:click="moveToConsultationRelatedInfoPage(consultationAsUser.consultation_id)" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">決済、返金、評価状況を確認する</button>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              相談は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationsAsUserErrMessage"/>
          </div> -->
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRoute } from 'vue-router'
import { Consultation } from '@/util/personalized/Consultation'

export default defineComponent({
  name: 'ConsultationRelatedInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const route = useRoute()
    const consultationId = route.params.consultation_id as string
    const outerErrorMessage = ref(null as string | null)

    const consultation = ref(null as Consultation | null)
    const consultationErrMessage = ref(null as string | null)

    onMounted(async () => {
      console.log('onMounted')
    })

    const requestsDone = computed(() => {
      return true
    })

    return {
      requestsDone,
      outerErrorMessage,
      consultation,
      consultationErrMessage
    }
  }
})
</script>
