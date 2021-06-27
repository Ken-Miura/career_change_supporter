<template>
  <div>
    <div class="container">
      <h1>登録済情報</h1>
      <p>{{profile.id}}</p>
      <p>{{profile.email}}</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Profile',
  setup () {
    const profile = reactive({
      email: '',
      name: '',
      furigana: '',
      telephoneNumber: '',
      dateOfBirth: '',
      address: ''
    })

    const router = useRouter()
    const store = useStore()
    // TODO: onMounted、onBeforeMount、setupのどれで呼ぶのが正しいか確認する
    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
        return
      }
      const response = await fetch('profile-information', {
        method: 'GET'
      })
      if (!response.ok) {
        // TODO: エラーハンドリング
        // profile.id = 'error: failed to get id'
        // profile.email = 'error: failed to get email'
        return
      }
      const userInfo = await response.json()
      profile.email = userInfo.email_address
      profile.name = userInfo.last_name + '　' + userInfo.first_name
      profile.furigana = userInfo.last_name_furigana + '　' + userInfo.first_name_furigana
      profile.dateOfBirth = userInfo.year + '年' + userInfo.month + '月' + userInfo.day + '日'
      profile.telephoneNumber = userInfo.telephone_number
      let address_line2 = '';
      if (userInfo.address_line2) {
        address_line2 = userInfo.address_line2
      } 
      // TODO: String.joinがない。。。
      profile.address = userInfo.prefecture + userInfo.city + userInfo.address_line1 + address_line2
    })
    return { profile }
  }
})
</script>

<style scoped>
.container {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
}
</style>
