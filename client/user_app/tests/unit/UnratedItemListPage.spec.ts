import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import UnratedItemListPage from '@/views/personalized/UnratedItemListPage.vue'
import { Message } from '@/util/Message'
import { ref } from 'vue'
import { UnratedItemsResultResp } from '@/util/personalized/unrated-item-list/UnratedItemsResultResp'
import { UnratedItemsResult } from '@/util/personalized/unrated-item-list/UnratedItemsResult'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { MAX_NUM_OF_UNRATED_CONSULTANTS, MAX_NUM_OF_UNRATED_USERS } from '@/util/personalized/unrated-item-list/MaxNumOfUnratedItems'
import { UnratedUser } from '@/util/personalized/unrated-item-list/UnratedUser'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'
import { UnratedConsultant } from '@/util/personalized/unrated-item-list/UnratedConsultant'

const getUnratedItemsDoneMock = ref(true)
const getUnratedItemsFuncMock = jest.fn()
jest.mock('@/util/personalized/unrated-item-list/useGetUnratedItems', () => ({
  useGetUnratedItems: () => ({
    getUnratedItemsDone: getUnratedItemsDoneMock,
    getUnratedItemsFunc: getUnratedItemsFuncMock
  })
}))

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

function createDummyUnratedConsultant1 (): UnratedConsultant {
  return {
    consultation_id: 234,
    consultant_id: 889,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 26,
      hour: 7
    } as ConsultationDateTime
  } as UnratedConsultant
}

function createDummyUnratedConsultant2 (): UnratedConsultant {
  return {
    consultation_id: 325,
    consultant_id: 1033,
    meeting_date_time_in_jst: {
      year: 2022,
      month: 12,
      day: 5,
      hour: 15
    } as ConsultationDateTime
  } as UnratedConsultant
}

function createDummyUnratedUser1 (): UnratedUser {
  return {
    consultation_id: 10,
    user_account_id: 53,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 26,
      hour: 7
    } as ConsultationDateTime
  } as UnratedUser
}

function createDummyUnratedUser2 (): UnratedUser {
  return {
    consultation_id: 11,
    user_account_id: 760,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 2,
      day: 24,
      hour: 15
    } as ConsultationDateTime
  } as UnratedUser
}

describe('UnratedItemListPage.vue', () => {
  beforeEach(() => {
    getUnratedItemsDoneMock.value = true
    getUnratedItemsFuncMock.mockReset()
    routerPushMock.mockClear()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getUnratedItemsDoneMock.value = false
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [], unrated_users: [] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
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

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getUnratedItemsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(UnratedItemListPage, {
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

  it(`moves to login if request returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getUnratedItemsFuncMock.mockResolvedValue(apiErrResp)
    mount(UnratedItemListPage, {
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

  it(`moves to login if request returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getUnratedItemsFuncMock.mockResolvedValue(apiErrResp)
    mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('displays no unrated consultants and unrated users when both do not exist', async () => {
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [], unrated_users: [] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)
    const noUnratedConsultantsLabel = wrapper.find('[data-test="no-unrated-consultants-label"]')
    expect(noUnratedConsultantsLabel.text()).toContain('未評価のコンサルタントはいません')

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noUnratedUsersLabel = wrapper.find('[data-test="no-unrated-users-label"]')
    expect(noUnratedUsersLabel.text()).toContain('未評価のユーザーはいません')
  })

  it('displays 1 unrated consultant and no unrated users', async () => {
    const dummyUnratedConsultant1 = createDummyUnratedConsultant1()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [dummyUnratedConsultant1], unrated_users: [] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)

    const unratedConsultant1 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant1.consultation_id}"]`)
    const consultantIdLabel1 = unratedConsultant1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUnratedConsultant1.consultant_id}）`)
    const unratedConsultant1ConsultationDateTime = unratedConsultant1.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant1.meeting_date_time_in_jst.year}年${dummyUnratedConsultant1.meeting_date_time_in_jst.month}月${dummyUnratedConsultant1.meeting_date_time_in_jst.day}日${dummyUnratedConsultant1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = unratedConsultant1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noUnratedUsersLabel = wrapper.find('[data-test="no-unrated-users-label"]')
    expect(noUnratedUsersLabel.text()).toContain('未評価のユーザーはいません')
  })

  it('displays 2 unrated consultants and no unrated users', async () => {
    const dummyUnratedConsultant1 = createDummyUnratedConsultant1()
    const dummyUnratedConsultant2 = createDummyUnratedConsultant2()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [dummyUnratedConsultant1, dummyUnratedConsultant2], unrated_users: [] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)

    const unratedConsultant1 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant1.consultation_id}"]`)
    const consultantIdLabel1 = unratedConsultant1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUnratedConsultant1.consultant_id}）`)
    const unratedConsultant1ConsultationDateTime = unratedConsultant1.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant1.meeting_date_time_in_jst.year}年${dummyUnratedConsultant1.meeting_date_time_in_jst.month}月${dummyUnratedConsultant1.meeting_date_time_in_jst.day}日${dummyUnratedConsultant1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = unratedConsultant1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const unratedConsultant2 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant2.consultation_id}"]`)
    const consultantIdLabel2 = unratedConsultant2.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel2.text()).toContain(`コンサルタントID（${dummyUnratedConsultant2.consultant_id}）`)
    const unratedConsultant2ConsultationDateTime = unratedConsultant2.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant2ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant2.meeting_date_time_in_jst.year}年${dummyUnratedConsultant2.meeting_date_time_in_jst.month}月${dummyUnratedConsultant2.meeting_date_time_in_jst.day}日${dummyUnratedConsultant2.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn2 = unratedConsultant2.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn2.exists()).toBe(true)

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)
    const noUnratedUsersLabel = wrapper.find('[data-test="no-unrated-users-label"]')
    expect(noUnratedUsersLabel.text()).toContain('未評価のユーザーはいません')
  })

  it('displays no unrated consultants and 1 unrated user', async () => {
    const dummyUnratedUser1 = createDummyUnratedUser1()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [], unrated_users: [dummyUnratedUser1] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)
    const noUnratedConsultantsLabel = wrapper.find('[data-test="no-unrated-consultants-label"]')
    expect(noUnratedConsultantsLabel.text()).toContain('未評価のコンサルタントはいません')

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const unratedUser1 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser1.consultation_id}"]`)
    const userAccountIdLabel1 = unratedUser1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyUnratedUser1.user_account_id}）`)
    const unratedUser1ConsultationDateTime = unratedUser1.find('[data-test="consultation-date-time"]')
    expect(unratedUser1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser1.meeting_date_time_in_jst.year}年${dummyUnratedUser1.meeting_date_time_in_jst.month}月${dummyUnratedUser1.meeting_date_time_in_jst.day}日${dummyUnratedUser1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = unratedUser1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)
  })

  it('displays no unrated consultants and 2 unrated users', async () => {
    const dummyUnratedUser1 = createDummyUnratedUser1()
    const dummyUnratedUser2 = createDummyUnratedUser2()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [], unrated_users: [dummyUnratedUser1, dummyUnratedUser2] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)
    const noUnratedConsultantsLabel = wrapper.find('[data-test="no-unrated-consultants-label"]')
    expect(noUnratedConsultantsLabel.text()).toContain('未評価のコンサルタントはいません')

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const unratedUser1 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser1.consultation_id}"]`)
    const userAccountIdLabel1 = unratedUser1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyUnratedUser1.user_account_id}）`)
    const unratedUser1ConsultationDateTime = unratedUser1.find('[data-test="consultation-date-time"]')
    expect(unratedUser1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser1.meeting_date_time_in_jst.year}年${dummyUnratedUser1.meeting_date_time_in_jst.month}月${dummyUnratedUser1.meeting_date_time_in_jst.day}日${dummyUnratedUser1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = unratedUser1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)

    const unratedUser2 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser2.consultation_id}"]`)
    const userAccountIdLabel2 = unratedUser2.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel2.text()).toContain(`ユーザーID（${dummyUnratedUser2.user_account_id}）`)
    const unratedUser2ConsultationDateTime = unratedUser2.find('[data-test="consultation-date-time"]')
    expect(unratedUser2ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser2.meeting_date_time_in_jst.year}年${dummyUnratedUser2.meeting_date_time_in_jst.month}月${dummyUnratedUser2.meeting_date_time_in_jst.day}日${dummyUnratedUser2.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn2 = unratedUser2.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn2.exists()).toBe(true)
  })

  it('displays 1 unrated consultant and 1 unrated user', async () => {
    const dummyUnratedConsultant1 = createDummyUnratedConsultant1()
    const dummyUnratedUser1 = createDummyUnratedUser1()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [dummyUnratedConsultant1], unrated_users: [dummyUnratedUser1] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)

    const unratedConsultant1 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant1.consultation_id}"]`)
    const consultantIdLabel1 = unratedConsultant1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUnratedConsultant1.consultant_id}）`)
    const unratedConsultant1ConsultationDateTime = unratedConsultant1.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant1.meeting_date_time_in_jst.year}年${dummyUnratedConsultant1.meeting_date_time_in_jst.month}月${dummyUnratedConsultant1.meeting_date_time_in_jst.day}日${dummyUnratedConsultant1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = unratedConsultant1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const unratedUser1 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser1.consultation_id}"]`)
    const userAccountIdLabel1 = unratedUser1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyUnratedUser1.user_account_id}）`)
    const unratedUser1ConsultationDateTime = unratedUser1.find('[data-test="consultation-date-time"]')
    expect(unratedUser1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser1.meeting_date_time_in_jst.year}年${dummyUnratedUser1.meeting_date_time_in_jst.month}月${dummyUnratedUser1.meeting_date_time_in_jst.day}日${dummyUnratedUser1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = unratedUser1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)
  })

  it('displays 2 unrated consultants and 2 unrated users', async () => {
    const dummyUnratedConsultant1 = createDummyUnratedConsultant1()
    const dummyUnratedConsultant2 = createDummyUnratedConsultant2()
    const dummyUnratedUser1 = createDummyUnratedUser1()
    const dummyUnratedUser2 = createDummyUnratedUser2()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [dummyUnratedConsultant1, dummyUnratedConsultant2], unrated_users: [dummyUnratedUser1, dummyUnratedUser2] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)

    const unratedConsultant1 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant1.consultation_id}"]`)
    const consultantIdLabel1 = unratedConsultant1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${dummyUnratedConsultant1.consultant_id}）`)
    const unratedConsultant1ConsultationDateTime = unratedConsultant1.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant1.meeting_date_time_in_jst.year}年${dummyUnratedConsultant1.meeting_date_time_in_jst.month}月${dummyUnratedConsultant1.meeting_date_time_in_jst.day}日${dummyUnratedConsultant1.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn1 = unratedConsultant1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)

    const unratedConsultant2 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant2.consultation_id}"]`)
    const consultantIdLabel2 = unratedConsultant2.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel2.text()).toContain(`コンサルタントID（${dummyUnratedConsultant2.consultant_id}）`)
    const unratedConsultant2ConsultationDateTime = unratedConsultant2.find('[data-test="consultation-date-time"]')
    expect(unratedConsultant2ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedConsultant2.meeting_date_time_in_jst.year}年${dummyUnratedConsultant2.meeting_date_time_in_jst.month}月${dummyUnratedConsultant2.meeting_date_time_in_jst.day}日${dummyUnratedConsultant2.meeting_date_time_in_jst.hour}時`)
    const moveToRateConsultantPageBtn2 = unratedConsultant2.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn2.exists()).toBe(true)

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const unratedUser1 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser1.consultation_id}"]`)
    const userAccountIdLabel1 = unratedUser1.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel1.text()).toContain(`ユーザーID（${dummyUnratedUser1.user_account_id}）`)
    const unratedUser1ConsultationDateTime = unratedUser1.find('[data-test="consultation-date-time"]')
    expect(unratedUser1ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser1.meeting_date_time_in_jst.year}年${dummyUnratedUser1.meeting_date_time_in_jst.month}月${dummyUnratedUser1.meeting_date_time_in_jst.day}日${dummyUnratedUser1.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn1 = unratedUser1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)

    const unratedUser2 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser2.consultation_id}"]`)
    const userAccountIdLabel2 = unratedUser2.find('[data-test="user-account-id-label"]')
    expect(userAccountIdLabel2.text()).toContain(`ユーザーID（${dummyUnratedUser2.user_account_id}）`)
    const unratedUser2ConsultationDateTime = unratedUser2.find('[data-test="consultation-date-time"]')
    expect(unratedUser2ConsultationDateTime.text()).toContain(`相談日時：${dummyUnratedUser2.meeting_date_time_in_jst.year}年${dummyUnratedUser2.meeting_date_time_in_jst.month}月${dummyUnratedUser2.meeting_date_time_in_jst.day}日${dummyUnratedUser2.meeting_date_time_in_jst.hour}時`)
    const moveToRateUserPageBtn2 = unratedUser2.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn2.exists()).toBe(true)
  })

  it('moves /rate-consultant with param when 評価する is clicked', async () => {
    const dummyUnratedConsultant1 = createDummyUnratedConsultant1()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [dummyUnratedConsultant1], unrated_users: [] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedConsultantsLabel = wrapper.find('[data-test="unrated-consultants-label"]')
    expect(unratedConsultantsLabel.text()).toContain('相談を行ったコンサルタント')
    const unratedConsultantsDescription = wrapper.find('[data-test="unrated-consultants-description"]')
    expect(unratedConsultantsDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_CONSULTANTS}件分表示されます。${MAX_NUM_OF_UNRATED_CONSULTANTS}件を超えた分は表示されているコンサルタントの評価を終えると表示されます。`)

    const unratedConsultant1 = wrapper.find(`[data-test="unrated-consultant-consultation-id-${dummyUnratedConsultant1.consultation_id}"]`)
    const moveToRateConsultantPageBtn1 = unratedConsultant1.find('[data-test="move-to-rate-consultant-page"]')
    expect(moveToRateConsultantPageBtn1.exists()).toBe(true)
    await moveToRateConsultantPageBtn1.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const dateTime = dummyUnratedConsultant1.meeting_date_time_in_jst
    const data = `/rate-consultant/${dummyUnratedConsultant1.consultation_id}?consultant-id=${dummyUnratedConsultant1.consultant_id}&year=${dateTime.year}&month=${dateTime.month}&day=${dateTime.day}&hour=${dateTime.hour}`
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })

  it('moves /rate-user with param when 評価する is clicked', async () => {
    const dummyUnratedUser1 = createDummyUnratedUser1()
    const resp = UnratedItemsResultResp.create({ unrated_consultants: [], unrated_users: [dummyUnratedUser1] } as UnratedItemsResult)
    getUnratedItemsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(UnratedItemListPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const unratedUsersLabel = wrapper.find('[data-test="unrated-users-label"]')
    expect(unratedUsersLabel.text()).toContain('相談を受け付けたユーザー')
    const unratedUsersDescription = wrapper.find('[data-test="unrated-users-description"]')
    expect(unratedUsersDescription.text()).toContain(`相談日時が古い方から最大${MAX_NUM_OF_UNRATED_USERS}件分表示されます。${MAX_NUM_OF_UNRATED_USERS}件を超えた分は表示されているユーザーの評価を終えると表示されます。`)

    const unratedUser1 = wrapper.find(`[data-test="unrated-user-consultation-id-${dummyUnratedUser1.consultation_id}"]`)
    const moveToRateUserPageBtn1 = unratedUser1.find('[data-test="move-to-rate-user-page"]')
    expect(moveToRateUserPageBtn1.exists()).toBe(true)
    await moveToRateUserPageBtn1.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const dateTime = dummyUnratedUser1.meeting_date_time_in_jst
    const data = `/rate-user/${dummyUnratedUser1.consultation_id}?user-id=${dummyUnratedUser1.user_account_id}&year=${dateTime.year}&month=${dateTime.month}&day=${dateTime.day}&hour=${dateTime.hour}`
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })
})
