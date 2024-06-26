import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import TheHeader from '@/components/TheHeader.vue'
import UserAccountInfoPage from '@/views/personalized/UserAccountInfoPage.vue'
import { UserAccountRetrievalResp } from '@/util/personalized/user-account-info/UserAccountRetrievalResp'
import { UserAccountRetrievalResult } from '@/util/personalized/user-account-info/UserAccountRetrievalResult'
import { UserAccount } from '@/util/personalized/user-account-info/UserAccount'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { GetAgreementsByUserAccountIdResp } from '@/util/personalized/user-account-info/terms-of-use/GetAgreementsByUserAccountIdResp'
import { AgreementsResult } from '@/util/personalized/user-account-info/terms-of-use/AgreementsResult'
import { GetIdentityOptionByUserAccountIdResp } from '@/util/personalized/user-account-info/identity/GetIdentityOptionByUserAccountIdResp'
import { IdentityResult } from '@/util/personalized/user-account-info/identity/IdentityResult'
import { GetCareersByUserAccountIdResp } from '@/util/personalized/user-account-info/career/GetCareersByUserAccountIdResp'
import { CareersResult } from '@/util/personalized/user-account-info/career/CareersResult'
import { GetFeePerHourInYenByUserAccountIdResp } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/GetFeePerHourInYenByUserAccountIdResp'
import { FeePerHourInYenResult } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/FeePerHourInYenResult'
import { GetConsultationReqsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByConsultantIdResp'
import { ConsultationReqsResult } from '@/util/personalized/user-account-info/consultation-req/ConsultationReqsResult'
import { GetConsultationReqsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByUserAccountIdResp'
import { GetRatingInfoByUserAccountIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByUserAccountIdResp'
import { RatingInfoResult } from '@/util/personalized/user-account-info/rating-info/RatingInfoResult'
import { GetRatingInfoByConsultantIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByConsultantIdResp'
import { GetIdentityCreationApprovalRecordResp } from '@/util/personalized/user-account-info/identity-creation/GetIdentityCreationApprovalRecordResp'
import { IdentityCreationApprovalRecordResult } from '@/util/personalized/user-account-info/identity-creation/IdentityCreationApprovalRecordResult'
import { GetIdentityCreationRejectionRecordsResp } from '@/util/personalized/user-account-info/identity-creation/GetIdentityCreationRejectionRecordsResp'
import { IdentityCreationRejectionRecordsResult } from '@/util/personalized/user-account-info/identity-creation/IdentityCreationRejectionRecordsResult'
import { GetIdentityUpdateApprovalRecordsResp } from '@/util/personalized/user-account-info/identity-update/GetIdentityUpdateApprovalRecordsResp'
import { IdentityUpdateApprovalRecordsResult } from '@/util/personalized/user-account-info/identity-update/IdentityUpdateApprovalRecordsResult'
import { GetIdentityUpdateRejectionRecordsResp } from '@/util/personalized/user-account-info/identity-update/GetIdentityUpdateRejectionRecordsResp'
import { IdentityUpdateRejectionRecordsResult } from '@/util/personalized/user-account-info/identity-update/IdentityUpdateRejectionRecordsResult'
import { GetCareerCreationRejectionRecordsResp } from '@/util/personalized/user-account-info/career-creation/GetCareerCreationRejectionRecordsResp'
import { CareerCreationRejectionRecordsResult } from '@/util/personalized/user-account-info/career-creation/CareerCreationRejectionRecordsResult'
import { GetCareerCreationApprovalRecordsResp } from '@/util/personalized/user-account-info/career-creation/GetCareerCreationApprovalRecordsResp'
import { CareerCreationApprovalRecordsResult } from '@/util/personalized/user-account-info/career-creation/CareerCreationApprovalRecordsResult'
import { GetConsultationsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByUserAccountIdResp'
import { ConsultationsResult } from '@/util/personalized/user-account-info/consultation/ConsultationsResult'
import { GetConsultationsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByConsultantIdResp'
import { GetBankAccountByUserAccountIdResp } from '@/util/personalized/user-account-info/bank-account/GetBankAccountByUserAccountIdResp'

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

const postUserAccountRetrievalDoneMock = ref(true)
const postUserAccountRetrievalByUserAccountIdFuncMock = jest.fn()
const postUserAccountRetrievalByEmailAddressFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/usePostUserAccountRetrieval', () => ({
  usePostUserAccountRetrieval: () => ({
    postUserAccountRetrievalDone: postUserAccountRetrievalDoneMock,
    postUserAccountRetrievalByUserAccountIdFunc: postUserAccountRetrievalByUserAccountIdFuncMock,
    postUserAccountRetrievalByEmailAddressFunc: postUserAccountRetrievalByEmailAddressFuncMock
  })
}))

const getAgreementsByUserAccountIdDoneMock = ref(true)
const getAgreementsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/terms-of-use/useGetAgreementsByUserAccountId', () => ({
  useGetAgreementsByUserAccountId: () => ({
    getAgreementsByUserAccountIdDone: getAgreementsByUserAccountIdDoneMock,
    getAgreementsByUserAccountIdFunc: getAgreementsByUserAccountIdFuncMock
  })
}))

const getIdentityOptionByUserAccountIdDoneMock = ref(true)
const getIdentityOptionByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity/useGetIdentityOptionByUserAccountId', () => ({
  useGetIdentityOptionByUserAccountId: () => ({
    getIdentityOptionByUserAccountIdDone: getIdentityOptionByUserAccountIdDoneMock,
    getIdentityOptionByUserAccountIdFunc: getIdentityOptionByUserAccountIdFuncMock
  })
}))

const getCareersByUserAccountIdDoneMock = ref(true)
const getCareersByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/career/useGetCareersByUserAccountId', () => ({
  useGetCareersByUserAccountId: () => ({
    getCareersByUserAccountIdDone: getCareersByUserAccountIdDoneMock,
    getCareersByUserAccountIdFunc: getCareersByUserAccountIdFuncMock
  })
}))

const getFeePerHourInYenByUserAccountIdDoneMock = ref(true)
const getFeePerHourInYenByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/fee-per-hour-in-yen/useGetFeePerHourInYenByUserAccountId', () => ({
  useGetFeePerHourInYenByUserAccountId: () => ({
    getFeePerHourInYenByUserAccountIdDone: getFeePerHourInYenByUserAccountIdDoneMock,
    getFeePerHourInYenByUserAccountIdFunc: getFeePerHourInYenByUserAccountIdFuncMock
  })
}))

const getBankAccountByUserAccountIdDoneMock = ref(true)
const getBankAccountByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/bank-account/useGetBankAccountByUserAccountId', () => ({
  useGetBankAccountByUserAccountId: () => ({
    getBankAccountByUserAccountIdDone: getBankAccountByUserAccountIdDoneMock,
    getBankAccountByUserAccountIdFunc: getBankAccountByUserAccountIdFuncMock
  })
}))

const getConsultationReqsByUserAccountIdDoneMock = ref(true)
const getConsultationReqsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByUserAccountId', () => ({
  useGetConsultationReqsByUserAccountId: () => ({
    getConsultationReqsByUserAccountIdDone: getConsultationReqsByUserAccountIdDoneMock,
    getConsultationReqsByUserAccountIdFunc: getConsultationReqsByUserAccountIdFuncMock
  })
}))

const getConsultationReqsByConsultantIdDoneMock = ref(true)
const getConsultationReqsByConsultantIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByConsultantId', () => ({
  useGetConsultationReqsByConsultantId: () => ({
    getConsultationReqsByConsultantIdDone: getConsultationReqsByConsultantIdDoneMock,
    getConsultationReqsByConsultantIdFunc: getConsultationReqsByConsultantIdFuncMock
  })
}))

const getConsultationsByUserAccountIdDoneMock = ref(true)
const getConsultationsByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation/useGetConsultationsByUserAccountId', () => ({
  useGetConsultationsByUserAccountId: () => ({
    getConsultationsByUserAccountIdDone: getConsultationsByUserAccountIdDoneMock,
    getConsultationsByUserAccountIdFunc: getConsultationsByUserAccountIdFuncMock
  })
}))

const getConsultationsByConsultantIdDoneMock = ref(true)
const getConsultationsByConsultantIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/consultation/useGetConsultationsByConsultantId', () => ({
  useGetConsultationsByConsultantId: () => ({
    getConsultationsByConsultantIdDone: getConsultationsByConsultantIdDoneMock,
    getConsultationsByConsultantIdFunc: getConsultationsByConsultantIdFuncMock
  })
}))

const getRatingInfoByUserAccountIdDoneMock = ref(true)
const getRatingInfoByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/rating-info/useGetRatingInfoByUserAccountId', () => ({
  useGetRatingInfoByUserAccountId: () => ({
    getRatingInfoByUserAccountIdDone: getRatingInfoByUserAccountIdDoneMock,
    getRatingInfoByUserAccountIdFunc: getRatingInfoByUserAccountIdFuncMock
  })
}))

const getRatingInfoByConsultantIdDoneMock = ref(true)
const getRatingInfoByConsultantIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/rating-info/useGetRatingInfoByConsultantId', () => ({
  useGetRatingInfoByConsultantId: () => ({
    getRatingInfoByConsultantIdDone: getRatingInfoByConsultantIdDoneMock,
    getRatingInfoByConsultantIdFunc: getRatingInfoByConsultantIdFuncMock
  })
}))

const getIdentityCreationApprovalRecordDoneMock = ref(true)
const getIdentityCreationApprovalRecordFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity-creation/useGetIdentityCreationApprovalRecord', () => ({
  useGetIdentityCreationApprovalRecord: () => ({
    getIdentityCreationApprovalRecordDone: getIdentityCreationApprovalRecordDoneMock,
    getIdentityCreationApprovalRecordFunc: getIdentityCreationApprovalRecordFuncMock
  })
}))

const getIdentityCreationRejectionRecordsDoneMock = ref(true)
const getIdentityCreationRejectionRecordsFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity-creation/useGetIdentityCreationRejectionRecords', () => ({
  useGetIdentityCreationRejectionRecords: () => ({
    getIdentityCreationRejectionRecordsDone: getIdentityCreationRejectionRecordsDoneMock,
    getIdentityCreationRejectionRecordsFunc: getIdentityCreationRejectionRecordsFuncMock
  })
}))

const getIdentityUpdateApprovalRecordsDoneMock = ref(true)
const getIdentityUpdateApprovalRecordsFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity-update/useGetIdentityUpdateApprovalRecords', () => ({
  useGetIdentityUpdateApprovalRecords: () => ({
    getIdentityUpdateApprovalRecordsDone: getIdentityUpdateApprovalRecordsDoneMock,
    getIdentityUpdateApprovalRecordsFunc: getIdentityUpdateApprovalRecordsFuncMock
  })
}))

const getIdentityUpdateRejectionRecordsDoneMock = ref(true)
const getIdentityUpdateRejectionRecordsFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/identity-update/useGetIdentityUpdateRejectionRecords', () => ({
  useGetIdentityUpdateRejectionRecords: () => ({
    getIdentityUpdateRejectionRecordsDone: getIdentityUpdateRejectionRecordsDoneMock,
    getIdentityUpdateRejectionRecordsFunc: getIdentityUpdateRejectionRecordsFuncMock
  })
}))

const getCareerCreationApprovalRecordsDoneMock = ref(true)
const getCareerCreationApprovalRecordsFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/career-creation/useGetCareerCreationApprovalRecords', () => ({
  useGetCareerCreationApprovalRecords: () => ({
    getCareerCreationApprovalRecordsDone: getCareerCreationApprovalRecordsDoneMock,
    getCareerCreationApprovalRecordsFunc: getCareerCreationApprovalRecordsFuncMock
  })
}))

const getCareerCreationRejectionRecordsDoneMock = ref(true)
const getCareerCreationRejectionRecordsFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/career-creation/useGetCareerCreationRejectionRecords', () => ({
  useGetCareerCreationRejectionRecords: () => ({
    getCareerCreationRejectionRecordsDone: getCareerCreationRejectionRecordsDoneMock,
    getCareerCreationRejectionRecordsFunc: getCareerCreationRejectionRecordsFuncMock
  })
}))

let userAccountSearchParamMock = null as UserAccountSearchParam | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      userAccountSearchParam: userAccountSearchParamMock
    }
  })
}))

function prepareInitValue () {
  const resp1 = UserAccountRetrievalResp.create({
    user_account: {
      user_account_id: 1,
      email_address: 'test0@test.com',
      last_login_time: '2023-04-13T14:12:53.4242+09:00',
      created_at: '2023-04-12T14:12:53.4242+09:00',
      mfa_enabled_at: null,
      disabled_at: null
    } as UserAccount
  } as UserAccountRetrievalResult)
  postUserAccountRetrievalByUserAccountIdFuncMock.mockResolvedValue(resp1)

  // postUserAccountRetrievalByUserAccountIdFuncを呼び出すのでこちらを使ったらエラーとする
  const errDetail = 'connection error'
  postUserAccountRetrievalByEmailAddressFuncMock.mockRejectedValue(new Error(errDetail))

  const resp2 = GetAgreementsByUserAccountIdResp.create({ agreements: [] } as AgreementsResult)
  getAgreementsByUserAccountIdFuncMock.mockResolvedValue(resp2)

  const resp3 = GetIdentityOptionByUserAccountIdResp.create({ identity_option: null } as IdentityResult)
  getIdentityOptionByUserAccountIdFuncMock.mockResolvedValue(resp3)

  const resp4 = GetCareersByUserAccountIdResp.create({ careers: [] } as CareersResult)
  getCareersByUserAccountIdFuncMock.mockResolvedValue(resp4)

  const resp5 = GetFeePerHourInYenByUserAccountIdResp.create({ fee_per_hour_in_yen: null } as FeePerHourInYenResult)
  getFeePerHourInYenByUserAccountIdFuncMock.mockResolvedValue(resp5)

  const resp6 = GetBankAccountByUserAccountIdResp.create(null)
  getBankAccountByUserAccountIdFuncMock.mockResolvedValue(resp6)

  const resp7 = GetConsultationReqsByUserAccountIdResp.create({ consultation_reqs: [] } as ConsultationReqsResult)
  getConsultationReqsByUserAccountIdFuncMock.mockResolvedValue(resp7)

  const resp8 = GetConsultationReqsByConsultantIdResp.create({ consultation_reqs: [] } as ConsultationReqsResult)
  getConsultationReqsByConsultantIdFuncMock.mockResolvedValue(resp8)

  const resp9 = GetConsultationsByUserAccountIdResp.create({ consultations: [] } as ConsultationsResult)
  getConsultationsByUserAccountIdFuncMock.mockResolvedValue(resp9)

  const resp10 = GetConsultationsByConsultantIdResp.create({ consultations: [] } as ConsultationsResult)
  getConsultationsByConsultantIdFuncMock.mockResolvedValue(resp10)

  const resp11 = GetRatingInfoByUserAccountIdResp.create({ average_rating: null, count: 0 } as RatingInfoResult)
  getRatingInfoByUserAccountIdFuncMock.mockResolvedValue(resp11)

  const resp12 = GetRatingInfoByConsultantIdResp.create({ average_rating: null, count: 0 } as RatingInfoResult)
  getRatingInfoByConsultantIdFuncMock.mockResolvedValue(resp12)

  const resp13 = GetIdentityCreationApprovalRecordResp.create({ approval_record: null } as IdentityCreationApprovalRecordResult)
  getIdentityCreationApprovalRecordFuncMock.mockResolvedValue(resp13)

  const resp14 = GetIdentityCreationRejectionRecordsResp.create({ rejection_records: [] } as IdentityCreationRejectionRecordsResult)
  getIdentityCreationRejectionRecordsFuncMock.mockResolvedValue(resp14)

  const resp15 = GetIdentityUpdateApprovalRecordsResp.create({ approval_records: [] } as IdentityUpdateApprovalRecordsResult)
  getIdentityUpdateApprovalRecordsFuncMock.mockResolvedValue(resp15)

  const resp16 = GetIdentityUpdateRejectionRecordsResp.create({ rejection_records: [] } as IdentityUpdateRejectionRecordsResult)
  getIdentityUpdateRejectionRecordsFuncMock.mockResolvedValue(resp16)

  const resp17 = GetCareerCreationApprovalRecordsResp.create({ approval_records: [] } as CareerCreationApprovalRecordsResult)
  getCareerCreationApprovalRecordsFuncMock.mockResolvedValue(resp17)

  const resp18 = GetCareerCreationRejectionRecordsResp.create({ rejection_records: [] } as CareerCreationRejectionRecordsResult)
  getCareerCreationRejectionRecordsFuncMock.mockResolvedValue(resp18)
}

describe('UserAccountInfoPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    postUserAccountRetrievalDoneMock.value = true
    postUserAccountRetrievalByUserAccountIdFuncMock.mockReset()
    postUserAccountRetrievalByEmailAddressFuncMock.mockReset()
    getAgreementsByUserAccountIdDoneMock.value = true
    getAgreementsByUserAccountIdFuncMock.mockReset()
    getIdentityOptionByUserAccountIdDoneMock.value = true
    getIdentityOptionByUserAccountIdFuncMock.mockReset()
    getCareersByUserAccountIdDoneMock.value = true
    getCareersByUserAccountIdFuncMock.mockReset()
    getFeePerHourInYenByUserAccountIdDoneMock.value = true
    getFeePerHourInYenByUserAccountIdFuncMock.mockReset()
    getBankAccountByUserAccountIdDoneMock.value = true
    getBankAccountByUserAccountIdFuncMock.mockReset()
    getConsultationReqsByUserAccountIdDoneMock.value = true
    getConsultationReqsByUserAccountIdFuncMock.mockReset()
    getConsultationReqsByConsultantIdDoneMock.value = true
    getConsultationReqsByConsultantIdFuncMock.mockReset()
    getConsultationsByUserAccountIdDoneMock.value = true
    getConsultationsByUserAccountIdFuncMock.mockReset()
    getConsultationsByConsultantIdDoneMock.value = true
    getConsultationsByConsultantIdFuncMock.mockReset()
    getRatingInfoByUserAccountIdDoneMock.value = true
    getRatingInfoByUserAccountIdFuncMock.mockReset()
    getRatingInfoByConsultantIdDoneMock.value = true
    getRatingInfoByConsultantIdFuncMock.mockReset()
    getIdentityCreationApprovalRecordDoneMock.value = true
    getIdentityCreationApprovalRecordFuncMock.mockReset()
    getIdentityCreationRejectionRecordsDoneMock.value = true
    getIdentityCreationRejectionRecordsFuncMock.mockReset()
    getIdentityUpdateApprovalRecordsDoneMock.value = true
    getIdentityUpdateApprovalRecordsFuncMock.mockReset()
    getIdentityUpdateRejectionRecordsDoneMock.value = true
    getIdentityUpdateRejectionRecordsFuncMock.mockReset()
    getCareerCreationApprovalRecordsDoneMock.value = true
    getCareerCreationApprovalRecordsFuncMock.mockReset()
    getCareerCreationRejectionRecordsDoneMock.value = true
    getCareerCreationRejectionRecordsFuncMock.mockReset()
    userAccountSearchParamMock = {
      accountId: 1,
      emailAddress: null
    } as UserAccountSearchParam
  })

  it('has WaitingCircle and TheHeader during postUserAccountRetrieval by user account id', async () => {
    prepareInitValue()
    postUserAccountRetrievalDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during postUserAccountRetrieval by email address', async () => {
    prepareInitValue()
    postUserAccountRetrievalDoneMock.value = false

    postUserAccountRetrievalByUserAccountIdFuncMock.mockReset()
    // postUserAccountRetrievalByEmailAddressFuncを呼び出すのでこちらを使ったらエラーとする
    const errDetail = 'connection error'
    postUserAccountRetrievalByUserAccountIdFuncMock.mockRejectedValue(new Error(errDetail))

    postUserAccountRetrievalByEmailAddressFuncMock.mockReset()
    const resp1 = UserAccountRetrievalResp.create({
      user_account: {
        user_account_id: 1,
        email_address: 'test0@test.com',
        last_login_time: '2023-04-13T14:12:53.4242+09:00',
        created_at: '2023-04-12T14:12:53.4242+09:00',
        mfa_enabled_at: null,
        disabled_at: null
      } as UserAccount
    } as UserAccountRetrievalResult)
    postUserAccountRetrievalByEmailAddressFuncMock.mockResolvedValue(resp1)

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getAgreementsByUserAccountId', async () => {
    prepareInitValue()
    getAgreementsByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityOptionByUserAccountId', async () => {
    prepareInitValue()
    getIdentityOptionByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getCareersByUserAccountId', async () => {
    prepareInitValue()
    getCareersByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getFeePerHourInYenByUserAccountId', async () => {
    prepareInitValue()
    getFeePerHourInYenByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getTenantIdByUserAccountId', async () => {
    prepareInitValue()
    getBankAccountByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getConsultationReqsByUserAccountId', async () => {
    prepareInitValue()
    getConsultationReqsByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getConsultationReqsByConsultantId', async () => {
    prepareInitValue()
    getConsultationReqsByConsultantIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getConsultationReqsByConsultantId', async () => {
    prepareInitValue()
    getConsultationReqsByConsultantIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getConsultationsByUserAccountId', async () => {
    prepareInitValue()
    getConsultationsByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getConsultationsByConsultantId', async () => {
    prepareInitValue()
    getConsultationsByConsultantIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getRatingInfoByUserAccountId', async () => {
    prepareInitValue()
    getRatingInfoByUserAccountIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getRatingInfoByConsultantId', async () => {
    prepareInitValue()
    getRatingInfoByConsultantIdDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityCreationApprovalRecord', async () => {
    prepareInitValue()
    getIdentityCreationApprovalRecordDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityCreationRejectionRecords', async () => {
    prepareInitValue()
    getIdentityCreationRejectionRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityUpdateApprovalRecords', async () => {
    prepareInitValue()
    getIdentityUpdateApprovalRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityUpdateRejectionRecords', async () => {
    prepareInitValue()
    getIdentityUpdateRejectionRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getIdentityUpdateRejectionRecords', async () => {
    prepareInitValue()
    getIdentityUpdateRejectionRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getCareerCreationApprovalRecords', async () => {
    prepareInitValue()
    getCareerCreationApprovalRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('has WaitingCircle and TheHeader during getCareerCreationRejectionRecords', async () => {
    prepareInitValue()
    getCareerCreationRejectionRecordsDoneMock.value = false

    const wrapper = mount(UserAccountInfoPage, {
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

  it('displays no error if all the requests are successful', async () => {
    prepareInitValue()

    const wrapper = mount(UserAccountInfoPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const waitingCircles = wrapper.findAllComponents(AlertMessage)
    expect(waitingCircles.length).toBe(0)
  })

  it('displays user account with user account id', async () => {
    prepareInitValue()

    const wrapper = mount(UserAccountInfoPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const srchCondLabel = wrapper.find('[data-test="search-condition-label"]')
    expect(srchCondLabel.text()).toContain('検索条件')
    const srchCondAccountId = wrapper.find('[data-test="search-condition-account-id"]')
    expect(srchCondAccountId.text()).toContain('アカウントID: 1')

    const accountInfoLabel = wrapper.find('[data-test="account-info-label"]')
    expect(accountInfoLabel.text()).toContain('アカウント情報')
    const accountId = wrapper.find('[data-test="account-id"]')
    expect(accountId.text()).toContain('アカウントID')
    const accountIdValue = wrapper.find('[data-test="account-id-value"]')
    expect(accountIdValue.text()).toContain('1')
    const emailAddress = wrapper.find('[data-test="email-address"]')
    expect(emailAddress.text()).toContain('メールアドレス')
    const emailAddressValue = wrapper.find('[data-test="email-address-value"]')
    expect(emailAddressValue.text()).toContain('test0@test.com')
  })

  // TODO: Add tests
})
