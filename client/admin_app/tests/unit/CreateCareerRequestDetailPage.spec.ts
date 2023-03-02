import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { nextTick, ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import CreateCareerRequestDetailPage from '@/views/personalized/CreateCareerRequestDetailPage.vue'
import { GetCreateCareerRequestDetailResp } from '@/util/personalized/create-career-request-detail/GetCreateCareerRequestDetailResp'
import { GetIdentityByUserAccountIdResp } from '@/util/personalized/GetIdentityByUserAccountIdResp'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Message } from '@/util/Message'
import AlertMessage from '@/components/AlertMessage.vue'
import { PostCreateCareerRequestApprovalResp } from '@/util/personalized/create-career-request-detail/PostCreateCareerRequestApprovalResp'

const routerPushMock = jest.fn()
let routeParam = ''
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      create_career_req_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const waitingGetCreateCareerRequestDetailDoneMock = ref(false)
const getCreateCareerRequestDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/create-career-request-detail/useGetCreateCareerRequestDetail', () => ({
  useGetCreateCareerRequestDetail: () => ({
    waitingGetCreateCareerRequestDetailDone: waitingGetCreateCareerRequestDetailDoneMock,
    getCreateCareerRequestDetailFunc: getCreateCareerRequestDetailFuncMock
  })
}))

const waitingGetIdentityByUserAccountIdDoneMock = ref(false)
const getIdentityByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/useGetIdentityByUserAccountId', () => ({
  useGetIdentityByUserAccountId: () => ({
    waitingGetIdentityByUserAccountIdDone: waitingGetIdentityByUserAccountIdDoneMock,
    getIdentityByUserAccountIdFunc: getIdentityByUserAccountIdFuncMock
  })
}))

const waitingPostCreateCareerRequestApprovalDoneMock = ref(false)
const postCreateCareerRequestApprovalFuncMock = jest.fn()
jest.mock('@/util/personalized/create-career-request-detail/usePostCreateCareerRequestApproval', () => ({
  usePostCreateCareerRequestApproval: () => ({
    waitingPostCreateCareerRequestApprovalDone: waitingPostCreateCareerRequestApprovalDoneMock,
    postCreateCareerRequestApprovalFunc: postCreateCareerRequestApprovalFuncMock
  })
}))

describe('CreateCareerRequestDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = ''
    routerPushMock.mockClear()
    waitingGetCreateCareerRequestDetailDoneMock.value = false
    getCreateCareerRequestDetailFuncMock.mockReset()
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    getIdentityByUserAccountIdFuncMock.mockReset()
    waitingPostCreateCareerRequestApprovalDoneMock.value = false
    postCreateCareerRequestApprovalFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader during getCreateCareerRequestDetail', async () => {
    routeParam = '1'
    waitingGetCreateCareerRequestDetailDoneMock.value = true
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    waitingPostCreateCareerRequestApprovalDoneMock.value = false
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    waitingGetCreateCareerRequestDetailDoneMock.value = false
    waitingGetIdentityByUserAccountIdDoneMock.value = true
    waitingPostCreateCareerRequestApprovalDoneMock.value = false
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    waitingGetCreateCareerRequestDetailDoneMock.value = false
    waitingGetIdentityByUserAccountIdDoneMock.value = false
    waitingPostCreateCareerRequestApprovalDoneMock.value = true
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} after getCreateCareerRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(apiErrResp)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    mount(CreateCareerRequestDetailPage, {
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
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(apiErrResp)
    mount(CreateCareerRequestDetailPage, {
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

  it(`displays ${Message.NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE} if ${Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND} after getCreateCareerRequestDetail`, async () => {
    routeParam = '1'
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND))
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(apiErrResp)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    expect(resultMessage).toContain(`${Message.NO_CREATE_CAREER_REQ_DETAIL_FOUND_MESSAGE} (${Code.NO_CREATE_CAREER_REQ_DETAIL_FOUND})`)
  })

  it('displays AlertMessage when error has happened during getCreateCareerRequestDetail', async () => {
    routeParam = '1'
    const errDetail = 'connection error'
    getCreateCareerRequestDetailFuncMock.mockRejectedValue(new Error(errDetail))
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
    const errDetail = 'connection error'
    getIdentityByUserAccountIdFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateCareerRequestDetailPage, {
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

  it('moves to CreateCareerRequestRejectionDetailPage if 拒否理由を選ぶ is pushed', async () => {
    routeParam = '1523'
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    const data = { name: 'CreateCareerRequestRejectionDetailPage', params: { create_career_req_id: routeParam } }
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('moves to create-career-request-approval if 承認する is pushed', async () => {
    routeParam = '1523'
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const resp3 = PostCreateCareerRequestApprovalResp.create()
    postCreateCareerRequestApprovalFuncMock.mockResolvedValue(resp3)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    expect(routerPushMock).toHaveBeenCalledWith('/create-career-request-approval')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned after pushing 承認する`, async () => {
    routeParam = '1523'
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postCreateCareerRequestApprovalFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const errDetail = 'connection error'
    postCreateCareerRequestApprovalFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: '開発部',
      office: '町田事業所',
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: {
        year: 2005,
        month: 3,
        day: 31
      },
      contract_type: 'contract',
      profession: 'エンジニア',
      annual_income_in_man_yen: 400,
      is_manager: true,
      position_name: '係長',
      is_new_graduate: true,
      note: '備考１',
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: 'd4df65633f6fa4ff2960000535156ece'
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
    const identity = {
      last_name: '田中',
      first_name: '太郎',
      last_name_furigana: 'タナカ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1987,
        month: 5,
        day: 21
      },
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: 'メゾンXXX　１０１',
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const wrapper = mount(CreateCareerRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const reqDetailDiv = wrapper.find('[data-test="req-detail"]')
    const reqDetail = reqDetailDiv.text()
    expect(reqDetail).toContain('職務経歴確認依頼詳細')
    expect(reqDetail).toContain('勤務先名称')
    expect(reqDetail).toContain(`${detail.company_name}`)
    expect(reqDetail).toContain('部署名')
    expect(reqDetail).toContain(`${detail.department_name}`)
    expect(reqDetail).toContain('勤務地')
    expect(reqDetail).toContain(`${detail.office}`)
    expect(reqDetail).toContain('入社日')
    expect(reqDetail).toContain(`${detail.career_start_date.year}年${detail.career_start_date.month}月${detail.career_start_date.day}日`)
    expect(reqDetail).toContain('退社日')
    expect(reqDetail).toContain(`${detail.career_end_date.year}年${detail.career_end_date.month}月${detail.career_end_date.day}日`)
    expect(reqDetail).toContain('雇用形態')
    expect(reqDetail).toContain('契約社員') // `${detail.contract_type}`
    expect(reqDetail).toContain('職種')
    expect(reqDetail).toContain(`${detail.profession}`)
    expect(reqDetail).toContain('年収（単位：万円）')
    expect(reqDetail).toContain(`${detail.annual_income_in_man_yen}`)
    expect(reqDetail).toContain('管理職区分')
    expect(reqDetail).toContain('管理職')
    expect(reqDetail).toContain('職位')
    expect(reqDetail).toContain(`${detail.position_name}`)
    expect(reqDetail).toContain('入社区分')
    expect(reqDetail).toContain('新卒入社') // `${detail.is_new_graduate}`
    expect(reqDetail).toContain('備考')
    expect(reqDetail).toContain(`${detail.note}`)
    expect(reqDetail).toContain('証明書画像（表面）')
    const image1Div = reqDetailDiv.find('[data-test="req-detail-image1"]')
    expect(image1Div.attributes().src).toBe(`/admin/api/career-images/${detail.user_account_id}/${detail.image1_file_name_without_ext}`)
    expect(reqDetail).toContain('証明書画像（裏面）')
    const image2Div = reqDetailDiv.find('[data-test="req-detail-image2"]')
    expect(image2Div.attributes().src).toBe(`/admin/api/career-images/${detail.user_account_id}/${detail.image2_file_name_without_ext}`)

    const identityDiv = wrapper.find('[data-test="identity"]')
    const identityDetail = identityDiv.text()
    expect(identityDetail).toContain('本人情報')
    expect(identityDetail).toContain('氏名が証明書画像の内容と一致しているか確認してください。')
    expect(identityDetail).toContain('氏名')
    expect(identityDetail).toContain(`${identity.last_name} ${identity.first_name}`)
    expect(identityDetail).toContain('フリガナ')
    expect(identityDetail).toContain(`${identity.last_name_furigana} ${identity.first_name_furigana}`)
    expect(identityDetail).toContain('生年月日')
    expect(identityDetail).toContain(`${identity.date_of_birth.year}年${identity.date_of_birth.month}月${identity.date_of_birth.day}日`)
    expect(identityDetail).toContain('住所')
    expect(identityDetail).toContain('都道府県')
    expect(identityDetail).toContain(`${identity.prefecture}`)
    expect(identityDetail).toContain('市区町村')
    expect(identityDetail).toContain(`${identity.city}`)
    expect(identityDetail).toContain('番地')
    expect(identityDetail).toContain(`${identity.address_line1}`)
    expect(identityDetail).toContain('建物名・部屋番号')
    expect(identityDetail).toContain(`${identity.address_line2}`)
    expect(identityDetail).toContain('電話番号')
    expect(identityDetail).toContain(`${identity.telephone_number}`)
  })

  it(`displays ${Message.NO_USER_ACCOUNT_FOUND_OR_THE_ACCOUNT_IS_DISABLED_MESSAGE} if ${Code.NO_USER_ACCOUNT_FOUND_OR_THE_ACCOUNT_IS_DISABLED} after 承認する is pushed`, async () => {
    routeParam = '1'
    const detail = {
      user_account_id: 705,
      company_name: 'テスト株式会社１',
      department_name: null,
      office: null,
      career_start_date: {
        year: 2002,
        month: 4,
        day: 1
      },
      career_end_date: null,
      contract_type: 'regular',
      profession: null,
      annual_income_in_man_yen: null,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: null,
      image1_file_name_without_ext: 'c9df65633f6fa4ff2960000535156eda',
      image2_file_name_without_ext: null
    }
    const resp1 = GetCreateCareerRequestDetailResp.create(detail)
    getCreateCareerRequestDetailFuncMock.mockResolvedValue(resp1)
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
      prefecture: '北海道',
      city: '札幌市',
      address_line1: '北区２−１',
      address_line2: null,
      telephone_number: '09012345678'
    }
    const resp2 = GetIdentityByUserAccountIdResp.create(identity)
    getIdentityByUserAccountIdFuncMock.mockResolvedValue(resp2)
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_USER_ACCOUNT_FOUND_OR_THE_ACCOUNT_IS_DISABLED))
    postCreateCareerRequestApprovalFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(CreateCareerRequestDetailPage, {
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
    expect(resultMessage).toContain(`${Message.NO_USER_ACCOUNT_FOUND_OR_THE_ACCOUNT_IS_DISABLED_MESSAGE} (${Code.NO_USER_ACCOUNT_FOUND_OR_THE_ACCOUNT_IS_DISABLED})`)
  })
})
