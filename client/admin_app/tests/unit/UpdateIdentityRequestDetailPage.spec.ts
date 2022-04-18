import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import UpdateIdentityRequestDetailPage from '@/views/personalized/UpdateIdentityRequestDetailPage.vue'
import { GetUpdateIdentityRequestDetailResp } from '@/util/personalized/update-identity-request-detail/GetUpdateIdentityRequestDetailResp'
import { GetIdentityByUserAccountIdResp } from '@/util/personalized/update-identity-request-detail/GetIdentityByUserAccountIdResp'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import AlertMessage from '@/components/AlertMessage.vue'
import { PostUpdateIdentityRequestApprovalResp } from '@/util/personalized/update-identity-request-detail/PostUpdateIdentityRequestApprovalResp'

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

const waitingGetUpdateIdentityRequestDetailDoneMock = ref(false)
const getUpdateIdentityRequestDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/update-identity-request-detail/useGetUpdateIdentityRequestDetail', () => ({
  useGetUpdateIdentityRequestDetail: () => ({
    waitingGetUpdateIdentityRequestDetailDone: waitingGetUpdateIdentityRequestDetailDoneMock,
    getUpdateIdentityRequestDetailFunc: getUpdateIdentityRequestDetailFuncMock
  })
}))

const waitingGetIdentityByUserAccountIdDoneMock = ref(false)
const getIdentityByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/update-identity-request-detail/useGetIdentityByUserAccountId', () => ({
  useGetIdentityByUserAccountId: () => ({
    waitingGetIdentityByUserAccountIdDone: waitingGetIdentityByUserAccountIdDoneMock,
    getIdentityByUserAccountIdFunc: getIdentityByUserAccountIdFuncMock
  })
}))

const waitingPostUpdateIdentityRequestApprovalDoneMock = ref(false)
const postUpdateIdentityRequestApprovalFuncMock = jest.fn()
jest.mock('@/util/personalized/update-identity-request-detail/usePostUpdateIdentityRequestApproval', () => ({
  usePostUpdateIdentityRequestApproval: () => ({
    waitingPostUpdateIdentityRequestApprovalDone: waitingPostUpdateIdentityRequestApprovalDoneMock,
    postUpdateIdentityRequestApprovalFunc: postUpdateIdentityRequestApprovalFuncMock
  })
}))

describe('UpdateIdentityRequestDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = ''
    routerPushMock.mockClear()
    waitingGetUpdateIdentityRequestDetailDoneMock.value = false
    getUpdateIdentityRequestDetailFuncMock.mockReset()
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    getIdentityByUserAccountIdFuncMock.mockReset()
    waitingPostUpdateIdentityRequestApprovalDoneMock.value = false
    postUpdateIdentityRequestApprovalFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader during getUpdateIdentityRequestDetail', async () => {
    routeParam = '1'
    waitingGetUpdateIdentityRequestDetailDoneMock.value = true
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    waitingPostUpdateIdentityRequestApprovalDoneMock.value = false
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityByUserAccountId', async () => {
    routeParam = '1'
    waitingGetUpdateIdentityRequestDetailDoneMock.value = false
    waitingGetIdentityByUserAccountIdDoneMock.value = true
    waitingPostUpdateIdentityRequestApprovalDoneMock.value = false
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
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

  it('has WaitingCircle and TheHeader during postUpdateIdentityRequestApproval', async () => {
    routeParam = '1'
    waitingGetUpdateIdentityRequestDetailDoneMock.value = false
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    waitingPostUpdateIdentityRequestApprovalDoneMock.value = true
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} after getUpdateIdentityRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} after getIdentityByUserAccountId`, async () => {
    routeParam = '1'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(apiErrResp)
    mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`displays ${Message.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} if ${Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND} after getUpdateIdentityRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND))
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_UPDATE_IDENTITY_REQ_DETAIL_FOUND})`)
  })

  it('displays AlertMessage when error has happened during getUpdateIdentityRequestDetail', async () => {
    routeParam = '1'
    const errDetail = 'connection error'
    getUpdateIdentityRequestDetailFuncMock.mockRejectedValue(new Error(errDetail))
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('displays AlertMessage when error has happened during getIdentityByUserAccountId', async () => {
    routeParam = '1'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const errDetail = 'connection error'
    getIdentityByUserAccountIdFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('moves to UpdateIdentityRequestRejectionDetailPage if 拒否理由を選ぶ is pushed', async () => {
    routeParam = '1523'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="choose-rejection-reason-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = { name: 'UpdateIdentityRequestRejectionDetailPage', params: { account_id: routeParam } }
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('moves to update-identity-request-approval if 承認する is pushed', async () => {
    routeParam = '1523'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const resp3 = PostUpdateIdentityRequestApprovalResp.create()
    postUpdateIdentityRequestApprovalFuncMock.mockResolvedValue(resp3)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="approve-req-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/update-identity-request-approval')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned after pushing 承認する`, async () => {
    routeParam = '1523'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postUpdateIdentityRequestApprovalFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="approve-req-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it('displays AlertMessage when error has happened after pushing 承認する', async () => {
    routeParam = '1523'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const errDetail = 'connection error'
    postUpdateIdentityRequestApprovalFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="approve-req-button"]')
    await button.trigger('click')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.UNEXPECTED_ERR)
    expect(resultMessage).toContain(errDetail)
  })

  it('displays request detail and identity', async () => {
    routeParam = '1626'
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
      address_line2: 'サーパスマンション１０１号',
      telephone_number: '09012345678',
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: 'cc22730f1780f733ca92e052260a9b15',
      requested_at: new Date(Date.UTC(2022, 4, 10, 16, 38, 43))
    }
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const reqDetailDiv = wrapper.find('[data-test="req-detail"]')
    const reqDetail = reqDetailDiv.text()
    expect(reqDetail).toContain('本人確認依頼（更新）詳細')
    expect(reqDetail).toContain('名前')
    expect(reqDetail).toContain(`${detail.last_name} ${detail.first_name}`)
    expect(reqDetail).toContain('フリガナ')
    expect(reqDetail).toContain(`${detail.last_name_furigana} ${detail.first_name_furigana}`)
    expect(reqDetail).toContain('生年月日')
    expect(reqDetail).toContain(`${detail.date_of_birth.year}年${detail.date_of_birth.month}月${detail.date_of_birth.day}日`)
    expect(reqDetail).toContain('住所')
    expect(reqDetail).toContain('都道府県')
    expect(reqDetail).toContain(`${detail.prefecture}`)
    expect(reqDetail).toContain('市区町村')
    expect(reqDetail).toContain(`${detail.city}`)
    expect(reqDetail).toContain('番地')
    expect(reqDetail).toContain(`${detail.address_line1}`)
    expect(reqDetail).toContain('建物名・部屋番号')
    expect(reqDetail).toContain(`${detail.address_line2}`)
    expect(reqDetail).toContain('電話番号')
    expect(reqDetail).toContain(`${detail.telephone_number}`)
    expect(reqDetail).toContain('身分証明書画像（表面）')
    const image1Div = reqDetailDiv.find('[data-test="req-detail-image1"]')
    expect(image1Div.attributes().src).toBe(`/admin/api/identity-images/${routeParam}/${detail.image1_file_name_without_ext}`)
    expect(reqDetail).toContain('身分証明書画像（裏面）')
    const image2Div = reqDetailDiv.find('[data-test="req-detail-image2"]')
    expect(image2Div.attributes().src).toBe(`/admin/api/identity-images/${routeParam}/${detail.image2_file_name_without_ext}`)
  })

  it(`displays ${Message.NO_USER_ACCOUNT_FOUND_MESSAGE} if ${Code.NO_USER_ACCOUNT_FOUND} after 承認する is pushed`, async () => {
    routeParam = '1'
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
    const resp1 = GetUpdateIdentityRequestDetailResp.create(detail)
    getUpdateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1994,
        month: 5,
        day: 21
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_USER_ACCOUNT_FOUND))
    postUpdateIdentityRequestApprovalFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(UpdateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const button = wrapper.find('[data-test="approve-req-button"]')
    await button.trigger('click')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.NO_USER_ACCOUNT_FOUND_MESSAGE} (${Code.NO_USER_ACCOUNT_FOUND})`)
  })
})
