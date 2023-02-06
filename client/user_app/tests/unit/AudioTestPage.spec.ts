import { ref } from 'vue'
import { getAudioMediaStream } from '@/util/personalized/AudioMediaStream'
import { generatePitchFactor } from '@/util/personalized/audio-test/PitchFacter'

const refreshDoneMock = ref(true)
const refreshFuncMock = jest.fn()
jest.mock('@/util/personalized/refresh/useRefresh', () => ({
  useRefresh: () => ({
    refreshDone: refreshDoneMock,
    refreshFunc: refreshFuncMock
  })
}))

jest.mock('@/util/personalized/AudioMediaStream')
const getAudioMediaStreamMock = getAudioMediaStream as jest.MockedFunction<typeof getAudioMediaStream>

jest.mock('@/util/personalized/audio-test/PitchFacter')
const generatePitchFactorMock = generatePitchFactor as jest.MockedFunction<typeof generatePitchFactor>

describe('AudioTestPage.vue', () => {
  beforeEach(() => {
    refreshDoneMock.value = true
    refreshFuncMock.mockReset()
    getAudioMediaStreamMock.mockReset()
    generatePitchFactorMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    console.log('test')
  })
})
