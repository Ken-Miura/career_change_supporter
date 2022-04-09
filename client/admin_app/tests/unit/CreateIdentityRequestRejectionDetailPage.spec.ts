import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateIdentityRequestRejectionDetailPage from '@/views/personalized/CreateIdentityRequestRejectionDetailPage.vue'
import { PostCreateIdentityRequestRejectionResp } from '@/util/personalized/create-identity-request-rejection-detail/PostCreateIdentityRequestRejectionResp'
import AlertMessage from '@/components/AlertMessage.vue'

const routerPushMock = jest.fn()
let routeParam = ''
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      account_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const waitingRequestDoneMock = ref(false)
const postCreateIdentityRequestRejectionFuncMock = jest.fn()
jest.mock('@/util/personalized/create-identity-request-rejection-detail/usePostCreateIdentityRequestRejection', () => ({
  usePostCreateIdentityRequestRejection: () => ({
    waitingRequestDone: waitingRequestDoneMock,
    postCreateIdentityRequestRejectionFunc: postCreateIdentityRequestRejectionFuncMock
  })
}))

describe('CreateIdentityRequestRejectionDetailPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    routeParam = ''
    waitingRequestDoneMock.value = false
    postCreateIdentityRequestRejectionFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    routeParam = '1'
    const resp = PostCreateIdentityRequestRejectionResp.create()
    postCreateIdentityRequestRejectionFuncMock.mockResolvedValue(resp)
    waitingRequestDoneMock.value = true
    const wrapper = mount(CreateIdentityRequestRejectionDetailPage, {
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

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(CreateIdentityRequestRejectionDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).toContain('hidden')
  })
})
