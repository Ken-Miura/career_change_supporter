import { mount, RouterLinkStub } from '@vue/test-utils'
import flushPromises from 'flush-promises'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import ConsultationRequestDetailPage from '@/views/personalized/ConsultationRequestDetailPage.vue'
import { ref } from 'vue'
import { GetConsultationRequestDetailResp } from '@/util/personalized/consultation-request-detail/GetConsultationRequestDetailResp'
import { ConsultationRequestDetail } from '@/util/personalized/consultation-request-detail/ConsultationRequestDetail'
import { ConsultationDateTime } from '@/util/personalized/ConsultationDateTime'
import { Message } from '@/util/Message'
import { Code } from '@/util/Error'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostConsultationRequestRejectionResp } from '@/util/personalized/consultation-request-detail/PostConsultationRequestRejectionResp'
import { PostConsultationRequestAcceptanceResp } from '@/util/personalized/consultation-request-detail/PostConsultationRequestAcceptanceResp'

let routeParam = ''
const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRoute: () => ({
    params: {
      consultation_req_id: routeParam
    }
  }),
  useRouter: () => ({
    push: routerPushMock
  })
}))

const getConsultationRequestDetailDoneMock = ref(true)
const getConsultationRequestDetailFuncMock = jest.fn()
jest.mock('@/util/personalized/consultation-request-detail/useGetConsultationRequestDetail', () => ({
  useGetConsultationRequestDetail: () => ({
    getConsultationRequestDetailDone: getConsultationRequestDetailDoneMock,
    getConsultationRequestDetailFunc: getConsultationRequestDetailFuncMock
  })
}))

const postConsultationRequestRejectionDoneMock = ref(true)
const postConsultationRequestRejectionFuncMock = jest.fn()
jest.mock('@/util/personalized/consultation-request-detail/usePostConsultationRequestRejection', () => ({
  usePostConsultationRequestRejection: () => ({
    postConsultationRequestRejectionDone: postConsultationRequestRejectionDoneMock,
    postConsultationRequestRejectionFunc: postConsultationRequestRejectionFuncMock
  })
}))

const postConsultationRequestAcceptanceDoneMock = ref(true)
const postConsultationRequestAcceptanceFuncMock = jest.fn()
jest.mock('@/util/personalized/consultation-request-detail/usePostConsultationRequestAcceptance', () => ({
  usePostConsultationRequestAcceptance: () => ({
    postConsultationRequestAcceptanceDone: postConsultationRequestAcceptanceDoneMock,
    postConsultationRequestAcceptanceFunc: postConsultationRequestAcceptanceFuncMock
  })
}))

function createDummyConsultationRequestDetail1 (consultationReq: number): ConsultationRequestDetail {
  return {
    consultation_req_id: consultationReq,
    user_account_id: 432,
    user_rating: null,
    num_of_rated_of_user: 0,
    fee_per_hour_in_yen: 7000,
    first_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 7
    } as ConsultationDateTime,
    second_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 8
    } as ConsultationDateTime,
    third_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 9
    } as ConsultationDateTime
  } as ConsultationRequestDetail
}

function createDummyConsultationRequestDetail2 (consultationReq: number): ConsultationRequestDetail {
  return {
    consultation_req_id: consultationReq,
    user_account_id: 432,
    user_rating: '3.8',
    num_of_rated_of_user: 567,
    fee_per_hour_in_yen: 7000,
    first_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 25,
      hour: 7
    } as ConsultationDateTime,
    second_candidate_in_jst: {
      year: 2023,
      month: 2,
      day: 1,
      hour: 17
    } as ConsultationDateTime,
    third_candidate_in_jst: {
      year: 2023,
      month: 1,
      day: 14,
      hour: 23
    } as ConsultationDateTime
  } as ConsultationRequestDetail
}

describe('ConsultationRequestDetailPage.vue', () => {
  beforeEach(() => {
    routeParam = '23'
    routerPushMock.mockClear()
    getConsultationRequestDetailDoneMock.value = true
    getConsultationRequestDetailFuncMock.mockReset()
    postConsultationRequestRejectionDoneMock.value = true
    postConsultationRequestRejectionFuncMock.mockReset()
    postConsultationRequestAcceptanceDoneMock.value = true
    postConsultationRequestAcceptanceFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while waiting response', async () => {
    getConsultationRequestDetailDoneMock.value = false
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
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
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
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

  it('displays AlertMessage when error has happened on opening ConsultationRequestDetailPage', async () => {
    const errDetail = 'connection error'
    getConsultationRequestDetailFuncMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(ConsultationRequestDetailPage, {
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
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestDetailPage, {
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
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    mount(ConsultationRequestDetailPage, {
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

  it(`displays ${Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE} if getting consultation req detail returns ${Code.NON_POSITIVE_CONSULTATION_REQ_ID}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NON_POSITIVE_CONSULTATION_REQ_ID))
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const errorMessage = wrapper.find('[data-test="error-message"]')
    expect(errorMessage.text()).toContain(Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE)
    expect(errorMessage.text()).toContain(Code.NON_POSITIVE_CONSULTATION_REQ_ID.toString())
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} if getting consultation req detail returns ${Code.NO_IDENTITY_REGISTERED}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const errorMessage = wrapper.find('[data-test="error-message"]')
    expect(errorMessage.text()).toContain(Message.NO_IDENTITY_REGISTERED_MESSAGE)
    expect(errorMessage.text()).toContain(Code.NO_IDENTITY_REGISTERED.toString())
  })

  it(`displays ${Message.NO_CONSULTATION_REQ_FOUND_MESSAGE} if getting consultation req detail returns ${Code.NO_CONSULTATION_REQ_FOUND}`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CONSULTATION_REQ_FOUND))
    getConsultationRequestDetailFuncMock.mockResolvedValue(apiErrResp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const errorMessage = wrapper.find('[data-test="error-message"]')
    expect(errorMessage.text()).toContain(Message.NO_CONSULTATION_REQ_FOUND_MESSAGE)
    expect(errorMessage.text()).toContain(Code.NO_CONSULTATION_REQ_FOUND.toString())
  })

  it('displays consultation request detail case 1', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultationReqDetailLabel = wrapper.find('[data-test="consultation-req-detail-label"]')
    expect(consultationReqDetailLabel.text()).toContain('相談申し込み詳細')
    const consultationReqDetailDescription = wrapper.find('[data-test="consultation-req-detail-description"]')
    expect(consultationReqDetailDescription.text()).toContain('詳細を確認し、相談申し込みを受けるかどうか選択して下さい。')
    const consulteeIdLabel = wrapper.find('[data-test="consultee-id-label"]')
    expect(consulteeIdLabel.text()).toContain('ユーザーID')
    const consulteeIdValue = wrapper.find('[data-test="consultee-id-value"]')
    expect(consulteeIdValue.text()).toContain(`${result.user_account_id}`)
    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain('評価')
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain(`0/5（評価件数：${result.num_of_rated_of_user} 件）`)
    const feeLabel = wrapper.find('[data-test="fee-label"]')
    expect(feeLabel.text()).toContain('相談料')
    const feeValue = wrapper.find('[data-test="fee-value"]')
    expect(feeValue.text()).toContain(`${result.fee_per_hour_in_yen} 円`)
    const candidatesLabel = wrapper.find('[data-test="candidates-label"]')
    expect(candidatesLabel.text()).toContain('希望相談開始日時候補一覧')
    const candidatesDescription = wrapper.find('[data-test="candidates-description"]')
    expect(candidatesDescription.text()).toContain('下記の候補一覧の内、一つを選択して下さい。相談は開始日時から1時間です。')
    const firstCandidateLabel = wrapper.find('[data-test="first-candidate-label"]')
    expect(firstCandidateLabel.text()).toContain(`第一希望: ${result.first_candidate_in_jst.year}年${result.first_candidate_in_jst.month}月${result.first_candidate_in_jst.day}日${result.first_candidate_in_jst.hour}時`)
    const secondCandidateLabel = wrapper.find('[data-test="second-candidate-label"]')
    expect(secondCandidateLabel.text()).toContain(`第二希望: ${result.second_candidate_in_jst.year}年${result.second_candidate_in_jst.month}月${result.second_candidate_in_jst.day}日${result.second_candidate_in_jst.hour}時`)
    const thirdCandidateLabel = wrapper.find('[data-test="third-candidate-label"]')
    expect(thirdCandidateLabel.text()).toContain(`第三希望: ${result.third_candidate_in_jst.year}年${result.third_candidate_in_jst.month}月${result.third_candidate_in_jst.day}日${result.third_candidate_in_jst.hour}時`)
    const confirmationLabel = wrapper.find('[data-test="confirmation-label"]')
    expect(confirmationLabel.text()).toContain('確認事項')
    const confirmationDescription = wrapper.find('[data-test="confirmation-description"]')
    expect(confirmationDescription.text()).toContain('相談申し込みを受け付けるためには、下記に記載の内容が正しいことを確認し、チェックをつけて下さい')
    const firstConfirmation = wrapper.find('[data-test="first-confirmation"]')
    expect(firstConfirmation.text()).toContain('私は社外秘とは何かを理解しており、それを口外することはありません。')
    const secondConfirmation = wrapper.find('[data-test="second-confirmation"]')
    expect(secondConfirmation.text()).toContain('私は相談申し込みを受けた後、それをキャンセルできないことを理解しています。')
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    expect(rejectBtn.text()).toContain('相談申し込みを拒否する')
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    expect(acceptBtn.text()).toContain('相談申し込みを受ける')
  })

  it('displays consultation request detail case 2', async () => {
    const result = createDummyConsultationRequestDetail2(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const consultationReqDetailLabel = wrapper.find('[data-test="consultation-req-detail-label"]')
    expect(consultationReqDetailLabel.text()).toContain('相談申し込み詳細')
    const consultationReqDetailDescription = wrapper.find('[data-test="consultation-req-detail-description"]')
    expect(consultationReqDetailDescription.text()).toContain('詳細を確認し、相談申し込みを受けるかどうか選択して下さい。')
    const consulteeIdLabel = wrapper.find('[data-test="consultee-id-label"]')
    expect(consulteeIdLabel.text()).toContain('ユーザーID')
    const consulteeIdValue = wrapper.find('[data-test="consultee-id-value"]')
    expect(consulteeIdValue.text()).toContain(`${result.user_account_id}`)
    const ratingLabel = wrapper.find('[data-test="rating-label"]')
    expect(ratingLabel.text()).toContain('評価')
    const ratingValue = wrapper.find('[data-test="rating-value"]')
    expect(ratingValue.text()).toContain(`${result.user_rating}/5（評価件数：${result.num_of_rated_of_user} 件）`)
    const feeLabel = wrapper.find('[data-test="fee-label"]')
    expect(feeLabel.text()).toContain('相談料')
    const feeValue = wrapper.find('[data-test="fee-value"]')
    expect(feeValue.text()).toContain(`${result.fee_per_hour_in_yen} 円`)
    const candidatesLabel = wrapper.find('[data-test="candidates-label"]')
    expect(candidatesLabel.text()).toContain('希望相談開始日時候補一覧')
    const candidatesDescription = wrapper.find('[data-test="candidates-description"]')
    expect(candidatesDescription.text()).toContain('下記の候補一覧の内、一つを選択して下さい。相談は開始日時から1時間です。')
    const firstCandidateLabel = wrapper.find('[data-test="first-candidate-label"]')
    expect(firstCandidateLabel.text()).toContain(`第一希望: ${result.first_candidate_in_jst.year}年${result.first_candidate_in_jst.month}月${result.first_candidate_in_jst.day}日${result.first_candidate_in_jst.hour}時`)
    const secondCandidateLabel = wrapper.find('[data-test="second-candidate-label"]')
    expect(secondCandidateLabel.text()).toContain(`第二希望: ${result.second_candidate_in_jst.year}年${result.second_candidate_in_jst.month}月${result.second_candidate_in_jst.day}日${result.second_candidate_in_jst.hour}時`)
    const thirdCandidateLabel = wrapper.find('[data-test="third-candidate-label"]')
    expect(thirdCandidateLabel.text()).toContain(`第三希望: ${result.third_candidate_in_jst.year}年${result.third_candidate_in_jst.month}月${result.third_candidate_in_jst.day}日${result.third_candidate_in_jst.hour}時`)
    const confirmationLabel = wrapper.find('[data-test="confirmation-label"]')
    expect(confirmationLabel.text()).toContain('確認事項')
    const confirmationDescription = wrapper.find('[data-test="confirmation-description"]')
    expect(confirmationDescription.text()).toContain('相談申し込みを受け付けるためには、下記に記載の内容が正しいことを確認し、チェックをつけて下さい')
    const firstConfirmation = wrapper.find('[data-test="first-confirmation"]')
    expect(firstConfirmation.text()).toContain('私は社外秘とは何かを理解しており、それを口外することはありません。')
    const secondConfirmation = wrapper.find('[data-test="second-confirmation"]')
    expect(secondConfirmation.text()).toContain('私は相談申し込みを受けた後、それをキャンセルできないことを理解しています。')
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    expect(rejectBtn.text()).toContain('相談申し込みを拒否する')
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    expect(acceptBtn.text()).toContain('相談申し込みを受ける')
  })

  it('has no error message below button on opening the page', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(false)
  })

  it('lets accept button disabled on opening the page', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    const buttonDisabledAttr = acceptBtn.attributes('disabled')
    expect(buttonDisabledAttr).toBeDefined()
  })

  it('has WaitingCircle and TheHeader while waiting response of rejection', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    postConsultationRequestRejectionDoneMock.value = false
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it('moves ConsultationRequestRejectionPage if reject button is pushed', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const rejectResp = PostConsultationRequestRejectionResp.create()
    postConsultationRequestRejectionFuncMock.mockResolvedValue(rejectResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultation-request-rejection')
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on rejection`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postConsultationRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms-of-use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on rejection`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postConsultationRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('displays AlertMessage when error has happened on rejection', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const errDetail = 'connection error'
    postConsultationRequestRejectionFuncMock.mockRejectedValue(new Error(errDetail))
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(true)
    expect(errorBelowBtn.text()).toContain(Message.UNEXPECTED_ERR)
    expect(errorBelowBtn.text()).toContain(errDetail)
  })

  it(`displays ${Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE} if ${Code.NON_POSITIVE_CONSULTATION_REQ_ID} is returned on rejection`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NON_POSITIVE_CONSULTATION_REQ_ID))
    postConsultationRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(true)
    expect(errorBelowBtn.text()).toContain(Message.NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE)
    expect(errorBelowBtn.text()).toContain(Code.NON_POSITIVE_CONSULTATION_REQ_ID.toString())
  })

  it(`displays ${Message.NO_IDENTITY_REGISTERED_MESSAGE} if ${Code.NO_IDENTITY_REGISTERED} is returned on rejection`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_REGISTERED))
    postConsultationRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(true)
    expect(errorBelowBtn.text()).toContain(Message.NO_IDENTITY_REGISTERED_MESSAGE)
    expect(errorBelowBtn.text()).toContain(Code.NO_IDENTITY_REGISTERED.toString())
  })

  it(`displays ${Message.NO_CONSULTATION_REQ_FOUND_MESSAGE} if ${Code.NO_CONSULTATION_REQ_FOUND} is returned on rejection`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_CONSULTATION_REQ_FOUND))
    postConsultationRequestRejectionFuncMock.mockResolvedValue(apiErrResp)
    const rejectBtn = wrapper.find('[data-test="reject-btn"]')
    await rejectBtn.trigger('click')
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(true)
    expect(errorBelowBtn.text()).toContain(Message.NO_CONSULTATION_REQ_FOUND_MESSAGE)
    expect(errorBelowBtn.text()).toContain(Code.NO_CONSULTATION_REQ_FOUND.toString())
  })

  it('has WaitingCircle and TheHeader while waiting response of acceptance', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    postConsultationRequestAcceptanceDoneMock.value = false
    await flushPromises()

    const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    expect(waitingCircles.length).toBe(1)
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it('moves ConsultationRequestAcceptancePage if first candidate is selected and then accept button is pushed', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidate = wrapper.find('[data-test="first-candidate"]')
    firstCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const acceptResp = PostConsultationRequestAcceptanceResp.create()
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(acceptResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 1, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultation-request-acceptance')
  })

  it('moves ConsultationRequestAcceptancePage if second candidate is selected and then accept button is pushed', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const secondCandidate = wrapper.find('[data-test="second-candidate"]')
    secondCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const acceptResp = PostConsultationRequestAcceptanceResp.create()
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(acceptResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 2, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultation-request-acceptance')
  })

  it('moves ConsultationRequestAcceptancePage if third candidate is selected and then accept button is pushed', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const thirdCandidate = wrapper.find('[data-test="third-candidate"]')
    thirdCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const acceptResp = PostConsultationRequestAcceptanceResp.create()
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(acceptResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 3, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultation-request-acceptance')
  })

  it('picks last selected candidate', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidate = wrapper.find('[data-test="first-candidate"]')
    firstCandidate.setValue(true)
    await flushPromises()
    const secondCandidate = wrapper.find('[data-test="second-candidate"]')
    secondCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const acceptResp = PostConsultationRequestAcceptanceResp.create()
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(acceptResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 2, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/consultation-request-acceptance')
  })

  it(`moves login if accept button is pushed and then ${Code.UNAUTHORIZED} is returned`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidate = wrapper.find('[data-test="first-candidate"]')
    firstCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(apiErrResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 1, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves terms-of-use if accept button is pushed and then ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned`, async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidate = wrapper.find('[data-test="first-candidate"]')
    firstCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postConsultationRequestAcceptanceFuncMock.mockResolvedValue(apiErrResp)
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 1, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it('displays AlertMessage when error has happened on acceptance', async () => {
    const result = createDummyConsultationRequestDetail1(parseInt(routeParam))
    const resp = GetConsultationRequestDetailResp.create(result)
    getConsultationRequestDetailFuncMock.mockResolvedValue(resp)
    const wrapper = mount(ConsultationRequestDetailPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const firstCandidate = wrapper.find('[data-test="first-candidate"]')
    firstCandidate.setValue(true)
    const userChecked = wrapper.find('[data-test="user-checked"]')
    userChecked.setValue(true)
    await flushPromises()

    const errDetail = 'connection error'
    postConsultationRequestAcceptanceFuncMock.mockRejectedValue(new Error(errDetail))
    const acceptBtn = wrapper.find('[data-test="accept-btn"]')
    await acceptBtn.trigger('click')
    await flushPromises()

    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{"consultation_req_id": ${result.consultation_req_id}, "picked_candidate": 1, "user_checked": true}`)
    expect(postConsultationRequestAcceptanceFuncMock).toHaveBeenCalledWith(data)
    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const errorBelowBtn = wrapper.find('[data-test="error-below-btn"]')
    expect(errorBelowBtn.exists()).toBe(true)
    expect(errorBelowBtn.text()).toContain(Message.UNEXPECTED_ERR)
    expect(errorBelowBtn.text()).toContain(errDetail)
  })
})
