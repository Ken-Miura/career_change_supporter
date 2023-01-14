import flushPromises from 'flush-promises'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ref } from 'vue'
import { RouterLinkStub, mount } from '@vue/test-utils'
import SchedulePage from '@/views/personalized/SchedulePage.vue'
import { GetConsultationsResp } from '@/util/personalized/schedule/GetConsultationsResp'
import { ConsultationsResult } from '@/util/personalized/schedule/ConsultationsResult'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UserSideConsultation } from '@/util/personalized/schedule/UserSideConsultation'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationsDoneMock = ref(true)
const getConsultationsFuncMock = jest.fn()
jest.mock('@/util/personalized/schedule/useGetConsultations', () => ({
  useGetConsultations: () => ({
    getConsultationsDone: getConsultationsDoneMock,
    getConsultationsFunc: getConsultationsFuncMock
  })
}))

function createDummyUserSideConsultation1 (): UserSideConsultation {
  return {
    consultation_id: 1,
    consultant_id: 1,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 1,
      day: 19,
      hour: 7
    } as ConsultationDateTime
  } as UserSideConsultation
}

function createDummyUserSideConsultation2 (): UserSideConsultation {
  return {
    consultation_id: 2,
    consultant_id: 2,
    meeting_date_time_in_jst: {
      year: 2023,
      month: 1,
      day: 20,
      hour: 23
    } as ConsultationDateTime
  } as UserSideConsultation
}

describe('SchedulePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    getConsultationsDoneMock.value = true
    getConsultationsFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while api call finishes', async () => {
    getConsultationsDoneMock.value = false
    const consultationsResult = {
      user_side_consultations: [],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
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

  it('has TheHeader, has no AlertMessage and WaitingCircle if request is done successfully', async () => {
    const consultationsResult = {
      user_side_consultations: [],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(0)
  })

  it('displays AlertMessage when error has happened', async () => {
    const errDetail = 'connection error'
    getConsultationsFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(SchedulePage, {
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

  it(`moves to login if getting consultation req detail returns ${Code.UNAUTHORIZED}`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getConsultationsFuncMock.mockResolvedValue(apiErrResp)
    mount(SchedulePage, {
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

  it(`moves to terms-of-use if getting consultation req detail returns ${Code.NOT_TERMS_OF_USE_AGREED_YET}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getConsultationsFuncMock.mockResolvedValue(apiErrResp)
    mount(SchedulePage, {
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

  it('displays no user side consultation and consultation description when both do not exist', async () => {
    const consultationsResult = {
      user_side_consultations: [],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userSideConsultationLabel = wrapper.find('[data-test="user-side-consultation-label"]')
    expect(userSideConsultationLabel.text()).toContain('あなたが申し込んだ相談')
    const noUserSideConsultationLabel = wrapper.find('[data-test="no-user-side-consultation-label"]')
    expect(noUserSideConsultationLabel.text()).toContain('あなたが申し込んだ相談はありません')

    const consultantSideConsultationLabel = wrapper.find('[data-test="consultant-side-consultation-label"]')
    expect(consultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談')
    const noConsultantSideConsultationLabel = wrapper.find('[data-test="no-consultant-side-consultation-label"]')
    expect(noConsultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談はありません')
  })

  it('displays 1 user side consultation and consultation description', async () => {
    const userDummy1 = createDummyUserSideConsultation1()
    const consultationsResult = {
      user_side_consultations: [userDummy1],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userSideConsultationLabel = wrapper.find('[data-test="user-side-consultation-label"]')
    expect(userSideConsultationLabel.text()).toContain('あなたが申し込んだ相談')
    const userSideConsultation1 = wrapper.find(`[data-test="user-side-consultation-id-${userDummy1.consultation_id}"]`)
    const consultantIdLabel1 = userSideConsultation1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${userDummy1.consultant_id}）への相談`)
    const userSideConsultationDateTime1 = userSideConsultation1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談開始日時：${userDummy1.meeting_date_time_in_jst.year}年${userDummy1.meeting_date_time_in_jst.month}月${userDummy1.meeting_date_time_in_jst.day}日${userDummy1.meeting_date_time_in_jst.hour}時`)

    const consultantSideConsultationLabel = wrapper.find('[data-test="consultant-side-consultation-label"]')
    expect(consultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談')
    const noConsultantSideConsultationLabel = wrapper.find('[data-test="no-consultant-side-consultation-label"]')
    expect(noConsultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談はありません')
  })

  it('displays 2 user side consultations and consultation description', async () => {
    const userDummy1 = createDummyUserSideConsultation1()
    const userDummy2 = createDummyUserSideConsultation2()
    const consultationsResult = {
      user_side_consultations: [userDummy1, userDummy2],
      consultant_side_consultations: []
    } as ConsultationsResult
    const resp = GetConsultationsResp.create(consultationsResult)
    getConsultationsFuncMock.mockResolvedValue(resp)
    const wrapper = mount(SchedulePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const userSideConsultationLabel = wrapper.find('[data-test="user-side-consultation-label"]')
    expect(userSideConsultationLabel.text()).toContain('あなたが申し込んだ相談')
    const userSideConsultation1 = wrapper.find(`[data-test="user-side-consultation-id-${userDummy1.consultation_id}"]`)
    const consultantIdLabel1 = userSideConsultation1.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel1.text()).toContain(`コンサルタントID（${userDummy1.consultant_id}）への相談`)
    const userSideConsultationDateTime1 = userSideConsultation1.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime1.text()).toContain(`相談開始日時：${userDummy1.meeting_date_time_in_jst.year}年${userDummy1.meeting_date_time_in_jst.month}月${userDummy1.meeting_date_time_in_jst.day}日${userDummy1.meeting_date_time_in_jst.hour}時`)
    const userSideConsultation2 = wrapper.find(`[data-test="user-side-consultation-id-${userDummy2.consultation_id}"]`)
    const consultantIdLabel2 = userSideConsultation2.find('[data-test="consultant-id-label"]')
    expect(consultantIdLabel2.text()).toContain(`コンサルタントID（${userDummy2.consultant_id}）への相談`)
    const userSideConsultationDateTime2 = userSideConsultation2.find('[data-test="user-side-consultation-date-time"]')
    expect(userSideConsultationDateTime2.text()).toContain(`相談開始日時：${userDummy2.meeting_date_time_in_jst.year}年${userDummy2.meeting_date_time_in_jst.month}月${userDummy2.meeting_date_time_in_jst.day}日${userDummy2.meeting_date_time_in_jst.hour}時`)

    const consultantSideConsultationLabel = wrapper.find('[data-test="consultant-side-consultation-label"]')
    expect(consultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談')
    const noConsultantSideConsultationLabel = wrapper.find('[data-test="no-consultant-side-consultation-label"]')
    expect(noConsultantSideConsultationLabel.text()).toContain('あなたが受け付けた相談はありません')
  })
})
