import { ref } from 'vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationsDoneMock = ref(true)
const getConsultationsFuncMock = jest.fn()
jest.mock('@/util/personalized/schedule/useGetConsultations', () => ({
  useGetConsultations: () => ({
    getConsultationsDone: getConsultationsDoneMock,
    getConsultationsFunc: getConsultationsFuncMock
  })
}))

describe('RewardPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getConsultationsDoneMock.value = true
    getConsultationsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while api call finishes', async () => {
    console.log('test')
  })
})
