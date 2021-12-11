<script lang="ts">
// TODO: Remove this component after @fullcalendar/vue for vue 3 is officially supported
// following code is inspired by https://github.com/fullcalendar/fullcalendar-vue/issues/131#issuecomment-773931889
// needed to make sure the `vdom` loads before everything else - fix for vite
import '@fullcalendar/core/vdom'
import { Calendar, CalendarOptions } from '@fullcalendar/core'
import {
  defineComponent,
  h,
  onMounted,
  onUnmounted,
  ref,
  watchEffect
} from 'vue'

export default defineComponent({
  props: {
    options: Object as () => CalendarOptions
  },

  setup (props) {
    const el = ref<HTMLElement>()
    const calendar = ref<Calendar>()

    onMounted(() => {
      if (el.value === undefined) {
        throw new ReferenceError('el.value is not defined')
      }
      calendar.value = new Calendar(el.value, props.options)
      calendar.value.render()
    })

    watchEffect(() => {
      if (calendar.value) {
        calendar.value.pauseRendering()
        calendar.value.resetOptions(props.options)
        calendar.value.resumeRendering()
      }
    })

    onUnmounted(() => {
      if (calendar.value === undefined) {
        console.warn('calendar.value is not defined')
        return
      }
      calendar.value.destroy()
    })

    return () => h('div', { ref: el })
  }
})
</script>
