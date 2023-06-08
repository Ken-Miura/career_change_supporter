import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { ref } from 'vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import TheHeader from '@/components/TheHeader.vue'
import UserAccountInfoPage from '@/views/personalized/UserAccountInfoPage.vue'
import { UserAccountRetrievalResp } from '@/util/personalized/user-account-info/UserAccountRetrievalResp'
import { UserAccountRetrievalResult } from '@/util/personalized/user-account-info/UserAccountRetrievalResult'
import { UserAccount } from '@/util/personalized/user-account-info/UserAccount'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'

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

const getTenantIdByUserAccountIdDoneMock = ref(true)
const getTenantIdByUserAccountIdFuncMock = jest.fn()
jest.mock('@/util/personalized/user-account-info/tenant/useGetTenantIdByUserAccountId', () => ({
  useGetTenantIdByUserAccountId: () => ({
    getTenantIdByUserAccountIdDone: getTenantIdByUserAccountIdDoneMock,
    getTenantIdByUserAccountIdFunc: getTenantIdByUserAccountIdFuncMock
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
    getTenantIdByUserAccountIdDoneMock.value = true
    getTenantIdByUserAccountIdFuncMock.mockReset()
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

  it('has WaitingCircle and TheHeader during Xxx', async () => {
    console.log('test')
    // const resp1 = UserAccountRetrievalResp.create({
    //   user_account: {
    //     user_account_id: 1,
    //     email_address: 'test0@test.com',
    //     last_login_time: '2023-04-13T14:12:53.4242+09:00',
    //     created_at: '2023-04-12T14:12:53.4242+09:00',
    //     mfa_enabled_at: null,
    //     disabled_at: null
    //   } as UserAccount
    // } as UserAccountRetrievalResult)
    // postUserAccountRetrievalByUserAccountIdFuncMock.mockResolvedValue(resp1)

    // const wrapper = mount(UserAccountInfoPage, {
    //   global: {
    //     stubs: {
    //       RouterLink: RouterLinkStub
    //     }
    //   }
    // })
    // await flushPromises()

    // const waitingCircles = wrapper.findAllComponents(WaitingCircle)
    // expect(waitingCircles.length).toBe(1)
    // const headers = wrapper.findAllComponents(TheHeader)
    // expect(headers.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })
})
