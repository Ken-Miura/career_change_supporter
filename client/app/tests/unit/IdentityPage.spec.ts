import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import IdentityPage from '@/views/personalized/IdentityPage.vue'
import { ref } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { Message } from '@/util/Message'
import { Identity } from '@/util/personalized/profile/Identity'

const waitingPostIdentityDoneMock = ref(false)
const postIdentityFuncMock = jest.fn()
jest.mock('@/util/personalized/identity/usePostIdentity', () => ({
  usePostIdentity: () => ({
    waitingPostIdentityDone: waitingPostIdentityDoneMock,
    postIdentityFunc: postIdentityFuncMock
  })
}))

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

let identityMock = null as Identity | null
const storeCommitMock = jest.fn()
jest.mock('vuex', () => ({
  useStore: () => ({
    commit: storeCommitMock,
    state: {
      identity: identityMock
    }
  })
}))

describe('IdentityPage.vue', () => {
  beforeEach(() => {
    waitingPostIdentityDoneMock.value = false
    postIdentityFuncMock.mockReset()
    refreshMock.mockReset()
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    identityMock = null
  })

  it('has one TheHeader, one submit button and one AlertMessage', () => {
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
    const submitButton = wrapper.find('[data-test="submit-button"]')
    expect(submitButton.exists)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
  })

  it('has labels for identity information input', () => {
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const lastName = wrapper.find('[data-test="last-name-div"]')
    expect(lastName.exists)
    expect(lastName.text()).toContain('姓')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    expect(firstName.exists)
    expect(firstName.text()).toContain('名')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    expect(lastNameFurigana.exists)
    expect(lastNameFurigana.text()).toContain('セイ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    expect(firstNameFurigana.exists)
    expect(firstNameFurigana.text()).toContain('メイ')
    const year = wrapper.find('[data-test="year-div"]')
    expect(year.exists)
    expect(year.text()).toContain('年')
    const month = wrapper.find('[data-test="month-div"]')
    expect(month.exists)
    expect(month.text()).toContain('月')
    const day = wrapper.find('[data-test="day-div"]')
    expect(day.exists)
    expect(day.text()).toContain('日')
    // 都道府県は、セレクトボックスのみでラベルはないのでチェックしない
    const city = wrapper.find('[data-test="city-div"]')
    expect(city.exists)
    expect(city.text()).toContain('市区町村')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    expect(addressLine1.exists)
    expect(addressLine1.text()).toContain('番地')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    expect(addressLine2.exists)
    expect(addressLine2.text()).toContain('建物名・部屋番号')
    const tel = wrapper.find('[data-test="tel-div"]')
    expect(tel.exists)
    expect(tel.text()).toContain('電話番号')
    const identityImage = wrapper.find('[data-test="identity-image-div"]')
    expect(identityImage.exists)
    expect(identityImage.text()).toContain('身分証明書')
    const identityImage1 = wrapper.find('[data-test="identity-image1-div"]')
    expect(identityImage1.exists)
    expect(identityImage1.text()).toContain('表面')
    const identityImage2 = wrapper.find('[data-test="identity-image2-div"]')
    expect(identityImage2.exists)
    expect(identityImage2.text()).toContain('裏面')
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(IdentityPage, {
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

  it('has TheHeader and WaitingCircle during api call', async () => {
    waitingPostIdentityDoneMock.value = true
    const wrapper = mount(IdentityPage, {
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
    expect(waitingCircles.length).toBe(1)
    // ユーザーに待ち時間を表すためにWaitingCircleが出ていることが確認できれば十分のため、
    // mainが出ていないことまで確認しない。
  })

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening IdentityPage`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('login')
  })

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening IdentityPage`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('terms-of-use')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened on opening IdentityPage`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(IdentityPage, {
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
})
