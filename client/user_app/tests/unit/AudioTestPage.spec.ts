import { ref } from 'vue'

// getAudioMediaStream mock
// generatePitchFactor mock

const refreshDoneMock = ref(true)
const refreshFuncMock = jest.fn()
jest.mock('@/util/personalized/refresh/useRefresh', () => ({
  useRefresh: () => ({
    refreshDone: refreshDoneMock,
    refreshFunc: refreshFuncMock
  })
}))

describe('AudioTestPage.vue', () => {
  beforeEach(() => {
    refreshDoneMock.value = true
    refreshFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    console.log('test')
  })
})
