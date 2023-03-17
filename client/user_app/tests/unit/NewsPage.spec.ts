import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import NewsPage from '@/views/NewsPage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { Message } from '@/util/Message'
import { GetNewResp } from '@/util/news/GetNewResp'
import { NewsResult } from '@/util/news/NewsResult'
import { News } from '@/util/news/News'
import { Ymd } from '@/util/Ymd'

const getNewsDoneMock = ref(true)
const getNewsFuncMock = jest.fn()
jest.mock('@/util/news/useGetNews', () => ({
  useGetNews: () => ({
    getNewsDone: getNewsDoneMock,
    getNewsFunc: getNewsFuncMock
  })
}))

const news1 = {
  news_id: 1,
  title: 'title1',
  body: `line1
  line2
  line2`,
  published_date_in_jst: {
    year: 2023,
    month: 3,
    day: 15
  } as Ymd
} as News

const news2 = {
  news_id: 2,
  title: 'title2',
  body: `a1
  b2
  c3
  d4`,
  published_date_in_jst: {
    year: 2023,
    month: 2,
    day: 20
  } as Ymd
} as News

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

  it('displays お知らせはありません when there is no news', async () => {
    const resp = GetNewResp.create({ news_array: [] } as NewsResult)
    getNewsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(NewsPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const newsLabel = wrapper.find('[data-test="news-label"]')
    expect(newsLabel.text()).toContain('お知らせ')
    const noNewsFound = wrapper.find('[data-test="no-news-found"]')
    expect(noNewsFound.text()).toContain('お知らせはありません')
  })

  it('displays 1 news', async () => {
    const resp = GetNewResp.create({ news_array: [news1] } as NewsResult)
    getNewsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(NewsPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const newsLabel = wrapper.find('[data-test="news-label"]')
    expect(newsLabel.text()).toContain('お知らせ')
    const noNewsFound = wrapper.find('[data-test="no-news-found"]')
    expect(noNewsFound.exists()).toBe(false)
    const news1Elem = wrapper.find(`[data-test="news-id-${news1.news_id}"]`)
    const title1 = news1Elem.find('[data-test="title"]')
    expect(title1.text()).toContain(`${news1.title}`)
    const body1 = news1Elem.find('[data-test="body"]')
    expect(body1.text()).toContain(`${news1.body}`)
  })
})
