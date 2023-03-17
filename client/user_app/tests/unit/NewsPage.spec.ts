import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import NewsPage from '@/views/NewsPage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'

const getNewsDoneMock = ref(true)
const getNewsFuncMock = jest.fn()
jest.mock('@/util/news/useGetNews', () => ({
  useGetNews: () => ({
    getNewsDone: getNewsDoneMock,
    getNewsFunc: getNewsFuncMock
  })
}))

describe('NewsPage.vue', () => {
  beforeEach(() => {
    getNewsDoneMock.value = true
    getNewsFuncMock.mockReset()
  })

  it('displays header and WaitingCircle, no AlertMessage while requesting', () => {
    getNewsDoneMock.value = false
    const wrapper = mount(NewsPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const headers = wrapper.findAll('header')
    expect(headers.length).toBe(1)
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getNewsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(NewsPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    expect(alertMessage).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })
})
