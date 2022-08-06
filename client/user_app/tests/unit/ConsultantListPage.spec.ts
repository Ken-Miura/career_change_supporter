import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import ConsultantListPage from '@/views/personalized/ConsultantListPage.vue'
import { PostConsultantsSearchResp } from '@/util/personalized/consultant-list/PostConsultantsSearchResp'
import { ConsultantsSearchResult } from '@/util/personalized/consultant-list/ConsultantsSearchResult'

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

jest.mock('@/util/PageSize')
const getPageSizeMock = getPageSize as jest.MockedFunction<typeof getPageSize>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const consultantSearchParamMock = null as ConsultantSearchParam | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      consultantSearchParam: consultantSearchParamMock
    }
  })
}))

const postConsultantsSearchDoneMock = ref(false)
const postConsultantsSearchFuncMock = jest.fn()
jest.mock('@/util/personalized/consultant-list/usePostConsultantsSearch', () => ({
  usePostConsultantsSearch: () => ({
    postConsultantsSearchDone: postConsultantsSearchDoneMock,
    postConsultantsSearchFunc: postConsultantsSearchFuncMock
  })
}))

describe('ConsultantListPage.vue', () => {
  beforeEach(() => {
    refreshMock.mockReset()
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
    postConsultantsSearchDoneMock.value = false
    postConsultantsSearchFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postConsultantsSearchDoneMock.value = false
    const result = {
      total: 0,
      consultants: []
    } as ConsultantsSearchResult
    const resp = PostConsultantsSearchResp.create(result)
    postConsultantsSearchFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultantListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })
})
