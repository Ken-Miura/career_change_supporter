import { ref } from 'vue'
import { postSetMaintenanceReq } from './PostSetMaintenanceReq'
import { SetMaintenanceReq } from './SetMaintenanceReq'

export function usePostSetMaintenanceReq () {
  const postSetMaintenanceReqDone = ref(true)
  const postSetMaintenanceReqFunc = async (req: SetMaintenanceReq) => {
    try {
      postSetMaintenanceReqDone.value = false
      const response = await postSetMaintenanceReq(req)
      return response
    } finally {
      postSetMaintenanceReqDone.value = true
    }
  }
  return {
    postSetMaintenanceReqDone,
    postSetMaintenanceReqFunc
  }
}
