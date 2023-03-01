<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <main class="flex flex-col justify-center bg-white max-w-lg mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
      <h3 class="font-bold text-lg">{{ message }}, {{ consultantRatingId }}, {{ userId }}, {{ year }}, {{ month }}, {{ day }}, {{ hour }}</h3>
    </main>
    <footer class="max-w-lg mx-auto flex justify-center text-white">
      <router-link to="/" class="hover:underline">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import { useRoute } from 'vue-router'

export default defineComponent({
  name: 'RateConsultantPage',
  components: {
    TheHeader
  },
  setup () {
    const message = 'RateConsultantPage'
    const route = useRoute()
    const consultantRatingId = route.params.consultant_rating_id as string
    const query = route.query
    const userId = query['consultant-id']
    const year = query.year
    const month = query.month
    const day = query.day
    const hour = query.hour

    return {
      message,
      consultantRatingId,
      userId,
      year,
      month,
      day,
      hour
    }
  }
})
</script>
