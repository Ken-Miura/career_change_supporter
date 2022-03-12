import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import IdentityPage from '@/views/personalized/IdentityPage.vue'
import { reactive, ref } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { Message } from '@/util/Message'
import { Identity } from '@/util/personalized/profile/Identity'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { PostIdentityResp } from '@/util/personalized/identity/PostIdentityResp'
import { SET_POST_IDENTITY_RESULT_MESSAGE } from '@/store/mutationTypes'
import { getMaxImageJpegImageSizeInBytes, MAX_JPEG_IMAGE_SIZE_IN_BYTES } from '@/util/MaxImageSize'

const waitingPostIdentityDoneMock = ref(false)
const postIdentityFuncMock = jest.fn()
jest.mock('@/util/personalized/identity/usePostIdentity', () => ({
  usePostIdentity: () => ({
    waitingPostIdentityDone: waitingPostIdentityDoneMock,
    postIdentityFunc: postIdentityFuncMock
  })
}))

let imagesMock = reactive({
  image1: null as File | null,
  image2: null as File | null
})
const onImage1StateChangeFuncMock = jest.fn()
const onImage2StateChangeFuncMock = jest.fn()
jest.mock('@/views/personalized/useImages', () => ({
  useImages: () => ({
    images: imagesMock,
    onImage1StateChange: onImage1StateChangeFuncMock,
    onImage2StateChange: onImage2StateChangeFuncMock
  })
}))

jest.mock('@/util/personalized/refresh/Refresh')
const refreshMock = refresh as jest.MockedFunction<typeof refresh>

jest.mock('@/util/MaxImageSize')
const getMaxImageJpegImageSizeInBytesMock = getMaxImageJpegImageSizeInBytes as jest.MockedFunction<typeof getMaxImageJpegImageSizeInBytes>

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

// 画像ファイルのモックは下記を参考に行う
// https://stackoverflow.com/questions/24488985/how-to-mock-file-in-javascript

describe('IdentityPage.vue', () => {
  beforeEach(() => {
    waitingPostIdentityDoneMock.value = false
    postIdentityFuncMock.mockReset()
    refreshMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(MAX_JPEG_IMAGE_SIZE_IN_BYTES)
    onImage1StateChangeFuncMock.mockReset()
    onImage2StateChangeFuncMock.mockReset()
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    identityMock = null
    imagesMock = reactive({
      image1: null as File | null,
      image2: null as File | null
    })
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

  it(`moves to post-identity-result setting ${Message.POST_IDENTITY_RESULT_MESSAGE} on store when postIdentity is success`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    const identity = {
      /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      date_of_birth: {
        year: 1990,
        month: 6,
        day: 14
      },
      prefecture: '東京都',
      city: '町田市',
      address_line1: '２−２−２２',
      address_line2: 'ライオンズマンション４０５',
      telephone_number: '08012345678'
      /* eslint-enable camelcase */
    }
    identityMock = identity
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image1.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: null as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('post-identity-result')
    expect(storeCommitMock).toHaveBeenCalledTimes(1)
    expect(storeCommitMock).toHaveBeenCalledWith(SET_POST_IDENTITY_RESULT_MESSAGE, `${Message.POST_IDENTITY_RESULT_MESSAGE}`)
  })

  // it(`moves to post-identity-result setting ${Message.POST_IDENTITY_RESULT_MESSAGE} on store when postIdentity is success from user input`, async () => {
  //   refreshMock.mockResolvedValue(RefreshResp.create())
  //   postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
  //   // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
  //   const image1 = new File(['test'], 'image1.jpeg', { type: 'image/jpeg' })
  //   imagesMock = reactive({
  //     image1: image1 as File | null,
  //     image2: null as File | null
  //   })
  //   getMaxImageJpegImageSizeInBytesMock.mockReset()
  //   getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size)
  //   const wrapper = mount(IdentityPage, {
  //     global: {
  //       stubs: {
  //         RouterLink: RouterLinkStub
  //       }
  //     }
  //   })
  //   const lastName = wrapper.find('[data-test="last-name-div"]')
  //   const lastNameInput = lastName.find('input')
  //   lastNameInput.setValue('山田')
  //   const firstName = wrapper.find('[data-test="first-name-div"]')
  //   const firstNameInput = firstName.find('input')
  //   firstNameInput.setValue('太郎')
  //   const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
  //   const lastNameFuriganaInput = lastNameFurigana.find('input')
  //   lastNameFuriganaInput.setValue('ヤマダ')
  //   const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
  //   const firstNameFuriganaInput = firstNameFurigana.find('input')
  //   firstNameFuriganaInput.setValue('太郎')
  //   const year = wrapper.find('[data-test="year-select-div"]')
  //   const yearOptions = wrapper.find('select').findAll('option')
  //   await yearOptions
  //   const month = wrapper.find('[data-test="month-div"]')
  //   expect(month.exists)
  //   expect(month.text()).toContain('月')
  //   const day = wrapper.find('[data-test="day-div"]')
  //   expect(day.exists)
  //   expect(day.text()).toContain('日')
  //   // 都道府県は、セレクトボックスのみでラベルはないのでチェックしない
  //   const city = wrapper.find('[data-test="city-div"]')
  //   expect(city.exists)
  //   expect(city.text()).toContain('市区町村')
  //   const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
  //   expect(addressLine1.exists)
  //   expect(addressLine1.text()).toContain('番地')
  //   const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
  //   expect(addressLine2.exists)
  //   expect(addressLine2.text()).toContain('建物名・部屋番号')
  //   const tel = wrapper.find('[data-test="tel-div"]')
  //   expect(tel.exists)
  //   expect(tel.text()).toContain('電話番号')
  //   const identityImage = wrapper.find('[data-test="identity-image-div"]')
  //   expect(identityImage.exists)
  //   expect(identityImage.text()).toContain('身分証明書')
  //   const identityImage1 = wrapper.find('[data-test="identity-image1-div"]')
  //   expect(identityImage1.exists)
  //   expect(identityImage1.text()).toContain('表面')
  //   const identityImage2 = wrapper.find('[data-test="identity-image2-div"]')
  //   expect(identityImage2.exists)
  //   expect(identityImage2.text()).toContain('裏面')

  //   await flushPromises()
  //   const submitButton = wrapper.find('[data-test="submit-button"]')
  //   await submitButton.trigger('submit')

  //   expect(routerPushMock).toHaveBeenCalledTimes(1)
  //   expect(routerPushMock).toHaveBeenCalledWith('post-identity-result')
  //   expect(storeCommitMock).toHaveBeenCalledTimes(1)
  //   expect(storeCommitMock).toHaveBeenCalledWith(SET_POST_IDENTITY_RESULT_MESSAGE, `${Message.POST_IDENTITY_RESULT_MESSAGE}`)
  // })
})
