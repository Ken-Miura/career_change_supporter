import { RouterLinkStub, mount } from '@vue/test-utils'
import flushPromises from 'flush-promises'
import RequestConsultationPage from '@/views/personalized/RequestConsultationPage.vue'
import { ref } from 'vue'
import { GetFeePerHourInYenForApplicationResp } from '@/util/personalized/request-consultation/GetFeePerHourInYenForApplicationResp'

let routeParam = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      consultant_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getFeePerHourInYenForApplicationDoneMock = ref(true)
const getFeePerHourInYenForApplicationFuncMock = jest.fn()
jest.mock('@/util/personalized/request-consultation/useGetFeePerHourInYenForApplication', () => ({
  useGetFeePerHourInYenForApplication: () => ({
    getFeePerHourInYenForApplicationDone: getFeePerHourInYenForApplicationDoneMock,
    getFeePerHourInYenForApplicationFunc: getFeePerHourInYenForApplicationFuncMock
  })
}))

const requestConsultationDoneMock = ref(true)
const startRequestConsultationMock = jest.fn()
const finishRequestConsultationMock = jest.fn()
const disabledMock = ref(true)
const disableBtnMock = jest.fn()
const enableBtnMock = jest.fn()
jest.mock('@/util/personalized/request-consultation/useRequestConsultationDone', () => ({
  useRequestConsultationDone: () => ({
    requestConsultationDone: requestConsultationDoneMock,
    startRequestConsultation: startRequestConsultationMock,
    finishRequestConsultation: finishRequestConsultationMock,
    disabled: disabledMock,
    disableBtn: disableBtnMock,
    enableBtn: enableBtnMock
  })
}))

// PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let payJpMock = null as any | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      payJp: payJpMock
    }
  })
}))

// PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const createPayJpMockObject = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  elements: (): any => {
    return {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      create: (type: string, options?: object): any => {
        console.log(`${type}, ${options}`)
        return {
          mount: (domElement: string) => {
            console.log(`${domElement}`)
          }
        }
      }
    }
  }
}
jest.mock('@/util/PayJp', () => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createPayJp: async (): Promise<any> => {
    return new Promise((resolve) => {
      resolve(createPayJpMockObject)
    })
  }
}))

describe('RequestConsultationPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    routerPushMock.mockClear()
    getFeePerHourInYenForApplicationDoneMock.value = true
    getFeePerHourInYenForApplicationFuncMock.mockReset()
    requestConsultationDoneMock.value = true
    startRequestConsultationMock.mockReset()
    finishRequestConsultationMock.mockReset()
    disabledMock.value = true
    disableBtnMock.mockReset()
    enableBtnMock.mockReset()
    payJpMock = null
  })

  it('has WaitingCircle and TheHeader while waiting response of fee per hour in yen', async () => {
    const resp = GetFeePerHourInYenForApplicationResp.create(5000)
    getFeePerHourInYenForApplicationFuncMock.mockResolvedValue(resp)
    const wrapper = mount(RequestConsultationPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
  })
})
