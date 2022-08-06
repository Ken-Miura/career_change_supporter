import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'
import { refresh } from '@/util/personalized/refresh/Refresh'
import { ref } from 'vue'

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

  it('', () => {
    console.log('TODO: Add test')
  })
})
