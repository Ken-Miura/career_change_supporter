<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 lg:pt-20 pb-6 px-2 lg:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!(getPlannedMaintenancesDone && postSetMaintenanceReqDone)" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">メンテナンスの計画</h3>
        <form @submit.prevent="setMaintenance">
          <div class="m-4 text-2xl grid grid-cols-6">
            <div class="mt-4 text-2xl justify-self-start col-span-6 pt-3">
              メンテナンス開始日時
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="startMtForm.year" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="startMtForm.month" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="startMtForm.day" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="startMtForm.hour" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="startMtForm.minute" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="minute in minuteList" v-bind:key="minute" v-bind:value="minute">{{ minute }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              分
            </div>
            <div class="mt-6 text-2xl justify-self-start col-span-6 pt-3">
              メンテナンス終了日時
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="endMtForm.year" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="year in yearList" v-bind:key="year" v-bind:value="year">{{ year }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              年
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="endMtForm.month" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="month in monthList" v-bind:key="month" v-bind:value="month">{{ month }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              月
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="endMtForm.day" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="day in dayList" v-bind:key="day" v-bind:value="day">{{ day }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              日
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="endMtForm.hour" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="hour in hourList" v-bind:key="hour" v-bind:value="hour">{{ hour }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              時
            </div>
            <div class="mt-2 w-full text-2xl justify-self-start col-span-5">
              <select v-model="endMtForm.minute" class="block w-full p-3 rounded-md shadow-sm focus:border-gray-700 focus:ring focus:ring-gray-300 focus:ring-opacity-50">
                <option v-for="minute in minuteList" v-bind:key="minute" v-bind:value="minute">{{ minute }}</option>
              </select>
            </div>
            <div class="mt-2 text-2xl justify-self-start col-span-1 pt-3 pl-3">
              分
            </div>
          </div>
          <div class="mt-6 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
            <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
              <div class="col-span-5">メンテナンス日時が適正であることを確認しました</div>
              <input v-model="setMaintenanceConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
            </div>
          </div>
          <button v-bind:disabled="!setMaintenanceConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none" type="submit">メンテナンスを設定する</button>
          <div v-if="setMaintenanceErrMessage" class="mt-6">
            <AlertMessage v-bind:message="setMaintenanceErrMessage"/>
          </div>
        </form>
      </div>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 lg:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">予定されているメンテナンス</h3>
        <div v-if="!plannedMaintenancesErrMessage">
            <div v-if="plannedMaintenances.length !== 0">
              <ul>
                <li v-for="p in plannedMaintenances" v-bind:key="p.maintenance_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">メンテナンス番号{{ p.maintenance_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">開始日時</div><div class="mt-2 justify-self-start col-span-2">{{ p.maintenance_start_at_in_jst }}</div>
                    <div class="mt-2 justify-self-start col-span-1">終了日時</div><div class="mt-2 justify-self-start col-span-2">{{ p.maintenance_end_at_in_jst }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              予定されているメンテナンスはありません。
            </div>
          </div>
        <div v-else>
          <AlertMessage class="mt-4" v-bind:message="plannedMaintenancesErrMessage"/>
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
import { defineComponent, onMounted, reactive, ref } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useRouter } from 'vue-router'
import { useGetPlannedMaintenances } from '@/util/personalized/planned-maintenance/useGetPlannedMaintenances'
import { PlannedMaintenance } from '@/util/personalized/planned-maintenance/PlannedMaintenance'
import { Code, createErrorMessage } from '@/util/Error'
import { ApiErrorResp } from '@/util/ApiError'
import { GetPlannedMaintenancesResp } from '@/util/personalized/planned-maintenance/GetPlannedMaintenancesResp'
import { Message } from '@/util/Message'
import { usePostSetMaintenanceReq } from '@/util/personalized/set-maintenance-req/usePostSetMaintenanceReq'
import { PostSetMaintenanceReqResp } from '@/util/personalized/set-maintenance-req/PostSetMaintenanceReqResp'
import { SetMaintenanceReq } from '@/util/personalized/set-maintenance-req/SetMaintenanceReq'
import { MaintenanceTime } from '@/util/personalized/set-maintenance-req/MaintenanceTime'

export default defineComponent({
  name: 'MaintenancesPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()

    const plannedMaintenances = ref([] as PlannedMaintenance[])
    const plannedMaintenancesErrMessage = ref(null as string | null)

    const {
      getPlannedMaintenancesDone,
      getPlannedMaintenancesFunc
    } = useGetPlannedMaintenances()

    const getPlannedMaintenances = async () => {
      try {
        const response = await getPlannedMaintenancesFunc()
        if (!(response instanceof GetPlannedMaintenancesResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          plannedMaintenancesErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getPlannedMaintenancesResult()
        plannedMaintenances.value = result.planned_maintenances
        plannedMaintenancesErrMessage.value = null
      } catch (e) {
        plannedMaintenancesErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await getPlannedMaintenances()
    })

    const currentDate = new Date()
    const yearList = ref([currentDate.getFullYear(), currentDate.getFullYear() + 1])
    const monthList = ref(createMonthList())
    const dayList = ref(createDayList())
    const hourList = ref(createHourList())
    const minuteList = ref(createMinuteList())

    const startMtForm = reactive({
      year: currentDate.getFullYear(),
      month: currentDate.getMonth() + 1,
      day: currentDate.getDate(),
      hour: currentDate.getHours(),
      minute: currentDate.getMinutes()
    })

    const endMtForm = reactive({
      year: currentDate.getFullYear(),
      month: currentDate.getMonth() + 1,
      day: currentDate.getDate(),
      hour: currentDate.getHours(),
      minute: currentDate.getMinutes()
    })

    const setMaintenanceErrMessage = ref(null as string | null)

    const {
      postSetMaintenanceReqDone,
      postSetMaintenanceReqFunc
    } = usePostSetMaintenanceReq()

    const setMaintenanceConfirmation = ref(false)

    const setMaintenance = async () => {
      const req = {
        start_time_in_jst: {
          year: startMtForm.year,
          month: startMtForm.month,
          day: startMtForm.day,
          hour: startMtForm.hour,
          minute: startMtForm.minute,
          second: 0
        } as MaintenanceTime,
        end_time_in_jst: {
          year: endMtForm.year,
          month: endMtForm.month,
          day: endMtForm.day,
          hour: endMtForm.hour,
          minute: endMtForm.minute,
          second: 0
        } as MaintenanceTime
      } as SetMaintenanceReq
      try {
        const response = await postSetMaintenanceReqFunc(req)
        if (!(response instanceof PostSetMaintenanceReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          setMaintenanceErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        setMaintenanceErrMessage.value = null
        await getPlannedMaintenances()
      } catch (e) {
        setMaintenanceErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        setMaintenanceConfirmation.value = false
      }
    }

    return {
      getPlannedMaintenancesDone,
      plannedMaintenances,
      plannedMaintenancesErrMessage,
      yearList,
      monthList,
      dayList,
      hourList,
      minuteList,
      setMaintenance,
      setMaintenanceErrMessage,
      postSetMaintenanceReqDone,
      startMtForm,
      endMtForm,
      setMaintenanceConfirmation
    }
  }
})

function createMonthList (): number[] {
  const months = [] as number[]
  for (let i = 0; i < 12; i++) {
    months.push(i + 1)
  }
  return months
}

function createDayList (): number[] {
  const days = [] as number[]
  for (let i = 0; i < 31; i++) {
    days.push(i + 1)
  }
  return days
}

function createHourList (): number[] {
  const days = [] as number[]
  for (let i = 0; i < 24; i++) {
    days.push(i)
  }
  return days
}

function createMinuteList (): number[] {
  const days = [] as number[]
  for (let i = 0; i < 60; i++) {
    days.push(i)
  }
  return days
}
</script>
