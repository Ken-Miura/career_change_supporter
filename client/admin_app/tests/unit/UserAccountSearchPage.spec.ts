import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { mount, RouterLinkStub } from '@vue/test-utils'
import TheHeader from '@/components/TheHeader.vue'
import UserAccountSearchPage from '@/views/personalized/UserAccountSearchPage.vue'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let userAccountSearchParamMock = null as UserAccountSearchParam | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      userAccountSearchParam: userAccountSearchParamMock
    }
  })
}))

describe('UserAccountSearchPage.vue', () => {
  beforeEach(() => {
    userAccountSearchParamMock = null
    storeCommitMock.mockClear()
    routerPushMock.mockClear()
  })

  it('diaplays header and contents for UserAccountSearchPage', async () => {
    const wrapper = mount(UserAccountSearchPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
  })
})
