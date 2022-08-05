import { getPageSize, PAGE_SIZE } from '@/util/PageSize'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'
import { refresh } from '@/util/personalized/refresh/Refresh'

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

describe('ConsultantListPage.vue', () => {
  beforeEach(() => {
    refreshMock.mockReset()
    getPageSizeMock.mockReset()
    getPageSizeMock.mockReturnValue(PAGE_SIZE)
    routerPushMock.mockClear()
  })

  it('', () => {
    console.log('TODO: Add test')
  })
})
