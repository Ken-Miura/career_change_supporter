import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateIdentityRequestDetailPage from '@/views/personalized/CreateIdentityRequestDetailPage.vue'
import { GetCreateIdentityRequestDetailResp } from '@/util/personalized/create-identity-request-detail/GetCreateIdentityRequestDetailResp'
import { GetUsersByDateOfBirthResp } from '@/util/personalized/create-identity-request-detail/GetUsersByDateOfBirthResp'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import AlertMessage from '@/components/AlertMessage.vue'
import { PostCreateIdentityRequestApprovalResp } from '@/util/personalized/create-identity-request-detail/PostCreateIdentityRequestApprovalResp'
import { AccountStatus } from '@/util/personalized/create-identity-request-detail/AccountStatus'

const routerPushMock = jest.fn()
let routeParam = ''
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      user_account_id: routeParam
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

describe('CreateIdentityRequestDetailPage.vue', () => {
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

  it(`moves to login if ${Code.UNAUTHORIZED} after getCreateIdentityRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    mount(CreateIdentityRequestDetailPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} after getUsersByDateOfBirth`, async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getUsersByDateOfBirthFuncMock.mockResolvedValue(apiErrResp)
    mount(CreateIdentityRequestDetailPage, {
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

  it(`displays ${Message.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} if ${Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND} after getCreateIdentityRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND))
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(apiErrResp)
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

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(`${Message.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_IDENTITY_REQ_DETAIL_FOUND})`)
  })

  it(`displays ${Message.ILLEGAL_DATE_MESSAGE} if ${Code.ILLEGAL_DATE} after getUsersByDateOfBirth`, async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_DATE))
    getUsersByDateOfBirthFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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
    expect(resultMessage).toContain(`${Message.ILLEGAL_DATE_MESSAGE} (${Code.ILLEGAL_DATE})`)
  })

  it('displays AlertMessage when error has happened during getCreateIdentityRequestDetail', async () => {
    routeParam = '1'
    const errDetail = 'connection error'
    getCreateIdentityRequestDetailFuncMock.mockRejectedValue(new Error(errDetail))
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

  it('displays AlertMessage when error has happened during getUsersByDateOfBirth', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const errDetail = 'connection error'
    getUsersByDateOfBirthFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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

  it('moves to CreateIdentityRequestRejectionDetailPage if 拒否理由を選ぶ is pushed', async () => {
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

    const button = wrapper.find('[data-test="choose-rejection-reason-button"]')
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = { name: 'CreateIdentityRequestRejectionDetailPage', params: { user_account_id: routeParam } }
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('moves to create-identity-request-approval if 承認する is pushed', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const resp3 = PostCreateIdentityRequestApprovalResp.create()
    postCreateIdentityRequestApprovalFuncMock.mockResolvedValue(resp3)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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
    expect(routerPushMock).toHaveBeenCalledWith('/create-identity-request-approval')
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postCreateIdentityRequestApprovalFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const resp2 = GetUsersByDateOfBirthResp.create([])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const errDetail = 'connection error'
    postCreateIdentityRequestApprovalFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateIdentityRequestDetailPage, {
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

  it('displays request detail', async () => {
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

    const reqDetailDiv = wrapper.find('[data-test="req-detail"]')
    const reqDetail = reqDetailDiv.text()
    expect(reqDetail).toContain('本人確認依頼（新規）詳細')
    expect(reqDetail).toContain('氏名')
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

  it('does not display any other users if same date of birth users is empty', async () => {
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

    const noUsersFoundDiv = wrapper.find('[data-test="no-same-date-of-birth-users-found"]')
    const noUsersFound = noUsersFoundDiv.text()
    expect(noUsersFound).toContain('生年月日が同じユーザーはいません。')
  })

  it('displays user who has same date of birth (account status is enabled)', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const user = {
      user_account_id: 5341,
      last_name: '佐藤',
      first_name: '次郎',
      last_name_furigana: 'サトウ',
      first_name_furigana: 'ジロウ',
      date_of_birth: detail.date_of_birth,
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: 'アーバンライフ２０２号',
      telephone_number: '07087654321',
      account_status: 'Enabled' as AccountStatus
    }
    const resp2 = GetUsersByDateOfBirthResp.create([user])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const usersDiv = wrapper.find('[data-test="same-date-of-birth-users"]')
    const users = usersDiv.text()
    expect(users).toContain(`ユーザーアカウントID: ${user.user_account_id}`)
    expect(users).toContain('アカウントステータス')
    expect(users).toContain('有効')
    expect(users).toContain('氏名')
    expect(users).toContain(`${user.last_name} ${user.first_name}`)
    expect(users).toContain('フリガナ')
    expect(users).toContain(`${user.last_name_furigana} ${user.first_name_furigana}`)
    expect(users).toContain('生年月日')
    expect(users).toContain(`${user.date_of_birth.year}年${user.date_of_birth.month}月${user.date_of_birth.day}日`)
    expect(users).toContain('住所')
    expect(users).toContain('都道府県')
    expect(users).toContain(`${user.prefecture}`)
    expect(users).toContain('市区町村')
    expect(users).toContain(`${user.city}`)
    expect(users).toContain('番地')
    expect(users).toContain(`${user.address_line1}`)
    expect(users).toContain('建物名・部屋番号')
    expect(users).toContain(`${user.address_line2}`)
    expect(users).toContain('電話番号')
    expect(users).toContain(`${user.telephone_number}`)
  })

  it('displays user who has same date of birth (account status is disabled)', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const user = {
      user_account_id: 5341,
      last_name: '佐藤',
      first_name: '次郎',
      last_name_furigana: 'サトウ',
      first_name_furigana: 'ジロウ',
      date_of_birth: detail.date_of_birth,
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: 'アーバンライフ２０２号',
      telephone_number: '07087654321',
      account_status: 'Disabled' as AccountStatus
    }
    const resp2 = GetUsersByDateOfBirthResp.create([user])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const usersDiv = wrapper.find('[data-test="same-date-of-birth-users"]')
    const users = usersDiv.text()
    expect(users).toContain(`ユーザーアカウントID: ${user.user_account_id}`)
    expect(users).toContain('アカウントステータス')
    expect(users).toContain('無効')
    expect(users).toContain('氏名')
    expect(users).toContain(`${user.last_name} ${user.first_name}`)
    expect(users).toContain('フリガナ')
    expect(users).toContain(`${user.last_name_furigana} ${user.first_name_furigana}`)
    expect(users).toContain('生年月日')
    expect(users).toContain(`${user.date_of_birth.year}年${user.date_of_birth.month}月${user.date_of_birth.day}日`)
    expect(users).toContain('住所')
    expect(users).toContain('都道府県')
    expect(users).toContain(`${user.prefecture}`)
    expect(users).toContain('市区町村')
    expect(users).toContain(`${user.city}`)
    expect(users).toContain('番地')
    expect(users).toContain(`${user.address_line1}`)
    expect(users).toContain('建物名・部屋番号')
    expect(users).toContain(`${user.address_line2}`)
    expect(users).toContain('電話番号')
    expect(users).toContain(`${user.telephone_number}`)
  })

  it('displays user who has same date of birth (account status is deleted)', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const user = {
      user_account_id: 5341,
      last_name: '佐藤',
      first_name: '次郎',
      last_name_furigana: 'サトウ',
      first_name_furigana: 'ジロウ',
      date_of_birth: detail.date_of_birth,
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: 'アーバンライフ２０２号',
      telephone_number: '07087654321',
      account_status: 'Deleted' as AccountStatus
    }
    const resp2 = GetUsersByDateOfBirthResp.create([user])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const usersDiv = wrapper.find('[data-test="same-date-of-birth-users"]')
    const users = usersDiv.text()
    expect(users).toContain(`ユーザーアカウントID: ${user.user_account_id}`)
    expect(users).toContain('アカウントステータス')
    expect(users).toContain('削除済み')
    expect(users).toContain('氏名')
    expect(users).toContain(`${user.last_name} ${user.first_name}`)
    expect(users).toContain('フリガナ')
    expect(users).toContain(`${user.last_name_furigana} ${user.first_name_furigana}`)
    expect(users).toContain('生年月日')
    expect(users).toContain(`${user.date_of_birth.year}年${user.date_of_birth.month}月${user.date_of_birth.day}日`)
    expect(users).toContain('住所')
    expect(users).toContain('都道府県')
    expect(users).toContain(`${user.prefecture}`)
    expect(users).toContain('市区町村')
    expect(users).toContain(`${user.city}`)
    expect(users).toContain('番地')
    expect(users).toContain(`${user.address_line1}`)
    expect(users).toContain('建物名・部屋番号')
    expect(users).toContain(`${user.address_line2}`)
    expect(users).toContain('電話番号')
    expect(users).toContain(`${user.telephone_number}`)
  })

  it('displays users who have same date of birth', async () => {
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
    const resp1 = GetCreateIdentityRequestDetailResp.create(detail)
    getCreateIdentityRequestDetailFuncMock.mockResolvedValue(resp1)
    const user1 = {
      user_account_id: 5341,
      last_name: '佐藤',
      first_name: '次郎',
      last_name_furigana: 'サトウ',
      first_name_furigana: 'ジロウ',
      date_of_birth: detail.date_of_birth,
      prefecture: '東京都',
      city: '町田市',
      address_line1: '森の里２−２２−２',
      address_line2: 'アーバンライフ２０２号',
      telephone_number: '07087654321',
      account_status: 'Enabled' as AccountStatus
    }
    const user2 = {
      user_account_id: 8785,
      last_name: '鈴木',
      first_name: '三郎',
      last_name_furigana: 'スズキ',
      first_name_furigana: 'サブロウ',
      date_of_birth: detail.date_of_birth,
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: 'アーバンライフ２０２号',
      telephone_number: '09087654321',
      account_status: 'Enabled' as AccountStatus
    }
    const resp2 = GetUsersByDateOfBirthResp.create([user1, user2])
    getUsersByDateOfBirthFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateIdentityRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const usersDiv = wrapper.find('[data-test="same-date-of-birth-users"]')
    const users = usersDiv.text()
    expect(users).toContain(`ユーザーアカウントID: ${user1.user_account_id}`)
    expect(users).toContain('氏名')
    expect(users).toContain(`${user1.last_name} ${user1.first_name}`)
    expect(users).toContain('フリガナ')
    expect(users).toContain(`${user1.last_name_furigana} ${user1.first_name_furigana}`)
    expect(users).toContain('生年月日')
    expect(users).toContain(`${user1.date_of_birth.year}年${user1.date_of_birth.month}月${user1.date_of_birth.day}日`)
    expect(users).toContain('住所')
    expect(users).toContain('都道府県')
    expect(users).toContain(`${user1.prefecture}`)
    expect(users).toContain('市区町村')
    expect(users).toContain(`${user1.city}`)
    expect(users).toContain('番地')
    expect(users).toContain(`${user1.address_line1}`)
    expect(users).toContain('建物名・部屋番号')
    expect(users).toContain(`${user1.address_line2}`)
    expect(users).toContain('電話番号')
    expect(users).toContain(`${user1.telephone_number}`)

    expect(users).toContain(`${user2.last_name} ${user2.first_name}`)
    expect(users).toContain(`${user2.last_name_furigana} ${user2.first_name_furigana}`)
    expect(users).toContain(`${user2.date_of_birth.year}年${user2.date_of_birth.month}月${user2.date_of_birth.day}日`)
    expect(users).toContain(`${user2.prefecture}`)
    expect(users).toContain(`${user2.city}`)
    expect(users).toContain(`${user2.address_line1}`)
    expect(users).toContain(`${user2.telephone_number}`)
  })
})
