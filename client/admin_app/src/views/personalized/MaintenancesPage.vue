<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!getPlannedMaintenancesDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <h3 class="font-bold text-2xl">予定されているメンテナンス</h3>
        <div v-if="!plannedMaintenancesErrMessage">
            <div v-if="plannedMaintenances.length !== 0">
              <ul>
                <li v-for="p in plannedMaintenances" v-bind:key="p.maintenance_id" class="mt-4">
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
import { defineComponent, onMounted, ref } from 'vue'
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
      } catch (e) {
        plannedMaintenancesErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      await getPlannedMaintenances()
    })

    return {
      getPlannedMaintenancesDone,
      plannedMaintenances,
      plannedMaintenancesErrMessage
    }
  }
})
</script>
