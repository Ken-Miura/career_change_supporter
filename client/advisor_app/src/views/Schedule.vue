<template>
  <FullCalendar :options="calendarOptions"/>
</template>

<script lang="ts">
import { defineComponent, onMounted } from 'vue'
import '@fullcalendar/core'
import dayGridPlugin from '@fullcalendar/daygrid'
import interactionPlugin from '@fullcalendar/interaction'
// TODO: Replace this component after @fullcalendar/vue for vue 3 is officially supported
import FullCalendar from '@/components/FullCalendar.vue'
import { useRouter } from 'vue-router'
import { getSessionState } from '@/store/SessionChecker'
import { useStore } from 'vuex'

export default defineComponent({
  name: 'Schedule',
  components: {
    FullCalendar
  },
  data () {
    return {
      calendarOptions: {
        plugins: [dayGridPlugin, interactionPlugin],
        initialView: 'dayGridMonth'
      }
    }
  },
  setup () {
    const router = useRouter()
    const store = useStore()

    onMounted(async () => {
      const sessionState = await getSessionState()
      store.commit('updateSessionState', sessionState)
      if (sessionState !== 'active') {
        await router.push('login')
      }
    })
  }
})
</script>
