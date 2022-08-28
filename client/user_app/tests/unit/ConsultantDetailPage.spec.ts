import { ref } from 'vue'

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

const getConsultantDetailDoneMock = ref(true)
const getConsultantDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/consultant-detail/useGetConsultantDetail', () => ({
  useGetConsultantDetail: () => ({
    getConsultantDetailDone: getConsultantDetailDoneMock,
    getConsultantDetailFunc: getConsultantDetailFuncMock
  })
}))

describe('ConsultantDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '1'
    routerPushMock.mockClear()
    getConsultantDetailDoneMock.value = true
    getConsultantDetailFuncMock.mockReset()
  })

  it('', async () => {
    console.log('test')
  })
})
