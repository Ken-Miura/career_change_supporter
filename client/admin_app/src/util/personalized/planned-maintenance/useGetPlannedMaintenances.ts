import { ref } from 'vue'
import { getPlannedMaintenances } from './GetPlannedMaintenances'

export function useGetPlannedMaintenances () {
  const getPlannedMaintenancesDone = ref(true)
  const getPlannedMaintenancesFunc = async () => {
    try {
      getPlannedMaintenancesDone.value = false
      const response = await getPlannedMaintenances()
      return response
    } finally {
      getPlannedMaintenancesDone.value = true
    }
  }
  return {
    getPlannedMaintenancesDone,
    getPlannedMaintenancesFunc
  }
}
