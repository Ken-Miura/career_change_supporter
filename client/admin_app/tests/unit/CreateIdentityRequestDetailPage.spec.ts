import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateIdentityRequestDetailPage from '@/views/personalized/CreateIdentityRequestDetailPage.vue'
import { GetCreateIdentityRequestDetailResp } from '@/util/personalized/create-identity-request-detail/GetCreateIdentityRequestDetailResp'
import { GetUsersByDateOfBirthResp } from '@/util/personalized/create-identity-request-detail/GetUsersByDateOfBirthResp'

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

const waitingGetCreateIdentityRequestDetailDoneMock = ref(false)
const getCreateIdentityRequestDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/create-identity-request-detail/useGetCreateIdentityRequestDetail', () => ({
  useGetCreateIdentityRequestDetail: () => ({
    waitingGetCreateIdentityRequestDetailDone: waitingGetCreateIdentityRequestDetailDoneMock,
    getCreateIdentityRequestDetailFunc: getCreateIdentityRequestDetailFuncMock
  })
}))

const waitingGetUsersByDateOfBirthDoneMock = ref(false)
const getUsersByDateOfBirthFuncMock = jest.fn()
jest.mock('@/util/personalized/create-identity-request-detail/useGetUsersByDateOfBirth', () => ({
  useGetUsersByDateOfBirth: () => ({
    waitingGetUsersByDateOfBirthDone: waitingGetUsersByDateOfBirthDoneMock,
    getUsersByDateOfBirthFunc: getUsersByDateOfBirthFuncMock
  })
}))

const waitingPostCreateIdentityRequestApprovalDoneMock = ref(false)
const postCreateIdentityRequestApprovalFuncMock = jest.fn()
jest.mock('@/util/personalized/create-identity-request-detail/usePostCreateIdentityRequestApproval', () => ({
  usePostCreateIdentityRequestApproval: () => ({
    waitingPostCreateIdentityRequestApprovalDone: waitingPostCreateIdentityRequestApprovalDoneMock,
    postCreateIdentityRequestApprovalFunc: postCreateIdentityRequestApprovalFuncMock
  })
}))

describe('CreateIdentityRequestRejectionDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = ''
    routerPushMock.mockClear()
    waitingGetCreateIdentityRequestDetailDoneMock.value = false
    getCreateIdentityRequestDetailFuncMock.mockReset()
    waitingGetUsersByDateOfBirthDoneMock.value = false
    getUsersByDateOfBirthFuncMock.mockReset()
    waitingPostCreateIdentityRequestApprovalDoneMock.value = false
    postCreateIdentityRequestApprovalFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader during getCreateIdentityRequestDetail', async () => {
    routeParam = '1'
    waitingGetCreateIdentityRequestDetailDoneMock.value = true
    waitingGetUsersByDateOfBirthDoneMock.value = false
    waitingPostCreateIdentityRequestApprovalDoneMock.value = false
    const detail = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678',
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null,
      requested_at: new Date(Date.UTC(2022, 4, 10, 16, 38, 43))
    }
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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

  it('has WaitingCircle and TheHeader during getUsersByDateOfBirth', async () => {
    routeParam = '1'
    waitingGetCreateIdentityRequestDetailDoneMock.value = false
    waitingGetUsersByDateOfBirthDoneMock.value = true
    waitingPostCreateIdentityRequestApprovalDoneMock.value = false
    const detail = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678',
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null,
      requested_at: new Date(Date.UTC(2022, 4, 10, 16, 38, 43))
    }
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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

  it('has WaitingCircle and TheHeader during postCreateIdentityRequestApproval', async () => {
    routeParam = '1'
    waitingGetCreateIdentityRequestDetailDoneMock.value = false
    waitingGetUsersByDateOfBirthDoneMock.value = false
    waitingPostCreateIdentityRequestApprovalDoneMock.value = true
    const detail = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678',
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null,
      requested_at: new Date(Date.UTC(2022, 4, 10, 16, 38, 43))
    }
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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
