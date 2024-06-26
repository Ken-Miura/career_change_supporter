import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import IdentityPage from '@/views/personalized/IdentityPage.vue'
import { nextTick, reactive, ref } from 'vue'
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
const resetImagesFuncMock = jest.fn()
jest.mock('@/views/personalized/useImages', () => ({
  useImages: () => ({
    images: imagesMock,
    onImage1StateChange: onImage1StateChangeFuncMock,
    onImage2StateChange: onImage2StateChangeFuncMock,
    resetImages: resetImagesFuncMock
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
    resetImagesFuncMock.mockReset()
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
    expect(submitButton.exists()).toBe(true)
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
    expect(lastName.exists()).toBe(true)
    expect(lastName.text()).toContain('姓')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    expect(firstName.exists()).toBe(true)
    expect(firstName.text()).toContain('名')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    expect(lastNameFurigana.exists()).toBe(true)
    expect(lastNameFurigana.text()).toContain('セイ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    expect(firstNameFurigana.exists()).toBe(true)
    expect(firstNameFurigana.text()).toContain('メイ')
    const year = wrapper.find('[data-test="year-div"]')
    expect(year.exists()).toBe(true)
    expect(year.text()).toContain('年')
    const month = wrapper.find('[data-test="month-div"]')
    expect(month.exists()).toBe(true)
    expect(month.text()).toContain('月')
    const day = wrapper.find('[data-test="day-div"]')
    expect(day.exists()).toBe(true)
    expect(day.text()).toContain('日')
    // 都道府県は、セレクトボックスのみでラベルはないのでチェックしない
    const city = wrapper.find('[data-test="city-div"]')
    expect(city.exists()).toBe(true)
    expect(city.text()).toContain('市区町村')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    expect(addressLine1.exists()).toBe(true)
    expect(addressLine1.text()).toContain('番地')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    expect(addressLine2.exists()).toBe(true)
    expect(addressLine2.text()).toContain('建物名・部屋番号')
    const tel = wrapper.find('[data-test="tel-div"]')
    expect(tel.exists()).toBe(true)
    expect(tel.text()).toContain('電話番号')
    const identityImage = wrapper.find('[data-test="identity-image-div"]')
    expect(identityImage.exists()).toBe(true)
    expect(identityImage.text()).toContain('身分証明書')
    const identityImage1 = wrapper.find('[data-test="identity-image1-div"]')
    expect(identityImage1.exists()).toBe(true)
    expect(identityImage1.text()).toContain('表面')
    const identityImage2 = wrapper.find('[data-test="identity-image2-div"]')
    expect(identityImage2.exists()).toBe(true)
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
    expect(routerPushMock).toHaveBeenCalledWith('/login')
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
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
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

  it('moves to submit-identity-success when postIdentity is success', async () => {
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
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-identity-success')
  })

  it('moves to submit-identity-success when postIdentity is success from user input', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-identity-success')
  })

  it(`displays alert message ${Message.NO_IDENTITY_IMAGE1_SELECTED} when image1 is not selected`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: null as File | null,
      image2: image2 as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image2.size)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_IMAGE1_SELECTED)
  })

  it(`displays alert message ${Message.NO_JPEG_EXTENSION_MESSAGE} when image1 file extension is not jpeg`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image.txt', { type: 'text/plain' })
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

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_JPEG_EXTENSION_MESSAGE)
  })

  it(`displays alert message ${Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE} when image1 exceeds max size`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: null as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size - 1)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE)
  })

  it(`displays alert message ${Message.NO_JPEG_EXTENSION_MESSAGE} when image2 file extension is not jpeg`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.txt', { type: 'text/plain' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_JPEG_EXTENSION_MESSAGE)
  })

  it(`displays alert message ${Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE} when image2 exceeds max size`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postIdentityFuncMock.mockResolvedValue(PostIdentityResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test_longer_binary_than_image1'], 'image.jpeg', { type: 'image/jpeg' })
    if (image2.size < image1.size) {
      throw new Error('image2.size < image1.size')
    }
    const maxImageSize = Math.max(image1.size, image2.size)
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize - 1)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE)
  })

  it(`moves to login when ${Code.UNAUTHORIZED} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    postIdentityFuncMock.mockRejectedValue(new Error(errDetail))
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
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

  it(`displays alert message ${Message.INVALID_LAST_NAME_LENGTH_MESSAGE} when last name length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_LAST_NAME_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_LAST_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_LAST_NAME_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE} when last name has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_LAST_NAME))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('\u000A')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_LAST_NAME.toString())
  })

  it(`displays alert message ${Message.INVALID_FIRST_NAME_LENGTH_MESSAGE} when first name length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_FIRST_NAME_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_FIRST_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_FIRST_NAME_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE} when first name has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_FIRST_NAME))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('\u000D')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_FIRST_NAME.toString())
  })

  it(`displays alert message ${Message.INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE} when last name furigana length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_LAST_NAME_FURIGANA_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_LAST_NAME_FURIGANA_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE} when last name furigana has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('\u0009')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA.toString())
  })

  it(`displays alert message ${Message.INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE} when first name furigana length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_FIRST_NAME_FURIGANA_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_FIRST_NAME_FURIGANA_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE} when first name furigana has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('\u0008')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_DATE_MESSAGE} when illgal date is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_DATE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('2')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('30')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_DATE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_DATE.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_AGE_MESSAGE} when user does not reach service-avalable age`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_AGE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    // 18歳以上であれば利用可能
    // 入力に限らずサーバから規定のエラーコードが返されるモックとしているので、こちらの値は適当に設定して良い。
    // ただ、基本的に仕様の理解のために適切な値を埋め込むようにする。
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('2022')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_AGE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_AGE.toString())
  })

  it(`displays alert message ${Message.INVALID_PREFECTURE_MESSAGE} when prefecture is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_PREFECTURE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('TOKYO')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_PREFECTURE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_PREFECTURE.toString())
  })

  it(`displays alert message ${Message.INVALID_CITY_LENGTH_MESSAGE} when city length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_CITY_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_CITY_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_CITY_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_CITY_MESSAGE} when city has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_CITY))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('\u007f')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_CITY_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_CITY.toString())
  })

  it(`displays alert message ${Message.INVALID_ADDRESS_LINE1_LENGTH_MESSAGE} when address line1 length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_ADDRESS_LINE1_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_ADDRESS_LINE1_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_ADDRESS_LINE1_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE} when address line1 has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE1))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('\u001b')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE1.toString())
  })

  it(`displays alert message ${Message.INVALID_ADDRESS_LINE2_LENGTH_MESSAGE} when address line2 length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_ADDRESS_LINE2_LENGTH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_ADDRESS_LINE2_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_ADDRESS_LINE2_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE} when address line2 has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE2))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('\u0007')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_ADDRESS_LINE2.toString())
  })

  it(`displays alert message ${Message.INVALID_TEL_NUM_FORMAT_MESSAGE} when tel format is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_TEL_NUM_FORMAT))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('090-1234-5678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_TEL_NUM_FORMAT_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_TEL_NUM_FORMAT.toString())
  })

  it(`displays alert message ${Message.NO_NAME_FOUND_MESSAGE} (invalid request case 1)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_NAME_FOUND))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_NAME_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_NAME_FOUND.toString())
  })

  it(`displays alert message ${Message.NO_FILE_NAME_FOUND_MESSAGE} (invalid request case 2)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_FILE_NAME_FOUND))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_FILE_NAME_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_FILE_NAME_FOUND.toString())
  })

  it(`displays alert message ${Message.DATA_PARSE_FAILURE_MESSAGE} (invalid request case 3)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.DATA_PARSE_FAILURE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.DATA_PARSE_FAILURE_MESSAGE)
    expect(resultMessage).toContain(Code.DATA_PARSE_FAILURE.toString())
  })

  it(`displays alert message ${Message.INVALID_NAME_IN_FIELD_MESSAGE} (invalid request case 4)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_NAME_IN_FIELD))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_NAME_IN_FIELD_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_NAME_IN_FIELD.toString())
  })

  it(`displays alert message ${Message.INVALID_UTF8_SEQUENCE_MESSAGE} (invalid request case 5)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_UTF8_SEQUENCE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_UTF8_SEQUENCE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_UTF8_SEQUENCE.toString())
  })

  it(`displays alert message ${Message.INVALID_IDENTITY_JSON_MESSAGE} (invalid request case 6)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_IDENTITY_JSON))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_IDENTITY_JSON_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_IDENTITY_JSON.toString())
  })

  it(`displays alert message ${Message.NO_JPEG_EXTENSION_MESSAGE} (invalid request case 7)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_JPEG_EXTENSION))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_JPEG_EXTENSION_MESSAGE)
    expect(resultMessage).toContain(Code.NO_JPEG_EXTENSION.toString())
  })

  it(`displays alert message ${Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE} (invalid request case 8)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE)
    expect(resultMessage).toContain(Code.EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT.toString())
  })

  it(`displays alert message ${Message.INVALID_JPEG_IMAGE_MESSAGE} (invalid request case 9)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_JPEG_IMAGE))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_JPEG_IMAGE_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_JPEG_IMAGE.toString())
  })

  it(`displays alert message ${Message.NO_IDENTITY_FOUND_MESSAGE} (invalid request case 10)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_FOUND))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_FOUND.toString())
  })

  it(`displays alert message ${Message.NO_IDENTITY_IMAGE1_FOUND_MESSAGE} (invalid request case 11)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_IMAGE1_FOUND))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_IMAGE1_FOUND_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_IMAGE1_FOUND.toString())
  })

  it(`displays alert message ${Message.IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE} (invalid request case 12)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.IDENTITY_INFO_REQ_ALREADY_EXISTS))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE)
    expect(resultMessage).toContain(Code.IDENTITY_INFO_REQ_ALREADY_EXISTS.toString())
  })

  it(`displays alert message ${Message.DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE} (invalid request case 13)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.DATE_OF_BIRTH_IS_NOT_MATCH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE)
    expect(resultMessage).toContain(Code.DATE_OF_BIRTH_IS_NOT_MATCH.toString())
  })

  it(`displays alert message ${Message.NO_IDENTITY_UPDATED_MESSAGE} (invalid request case 14)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_IDENTITY_UPDATED))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_IDENTITY_UPDATED_MESSAGE)
    expect(resultMessage).toContain(Code.NO_IDENTITY_UPDATED.toString())
  })

  it(`displays alert message ${Message.FIRST_NAME_IS_NOT_MATCH_MESSAGE} (invalid request case 15)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.FIRST_NAME_IS_NOT_MATCH))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.FIRST_NAME_IS_NOT_MATCH_MESSAGE)
    expect(resultMessage).toContain(Code.FIRST_NAME_IS_NOT_MATCH.toString())
  })

  it(`displays alert message ${Message.INVALID_MULTIPART_FORM_DATA_MESSAGE} (invalid request case 15)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_MULTIPART_FORM_DATA))
    postIdentityFuncMock.mockResolvedValue(apiErrResp)
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test1'], 'image1.jpeg', { type: 'image/jpeg' })
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: image2 as File | null
    })
    const maxImageSize = Math.max(image1.size, image2.size)
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(maxImageSize)
    const wrapper = mount(IdentityPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const lastName = wrapper.find('[data-test="last-name-div"]')
    const lastNameInput = lastName.find('input')
    await lastNameInput.setValue('山田')
    const firstName = wrapper.find('[data-test="first-name-div"]')
    const firstNameInput = firstName.find('input')
    await firstNameInput.setValue('太郎')
    const lastNameFurigana = wrapper.find('[data-test="last-name-furigana-div"]')
    const lastNameFuriganaInput = lastNameFurigana.find('input')
    await lastNameFuriganaInput.setValue('ヤマダ')
    const firstNameFurigana = wrapper.find('[data-test="first-name-furigana-div"]')
    const firstNameFuriganaInput = firstNameFurigana.find('input')
    await firstNameFuriganaInput.setValue('タロウ')
    const year = wrapper.find('[data-test="year-select-div"]')
    const yearSelect = year.find('select')
    await yearSelect.setValue('1990')
    const month = wrapper.find('[data-test="month-select-div"]')
    const monthSelect = month.find('select')
    await monthSelect.setValue('5')
    const day = wrapper.find('[data-test="day-select-div"]')
    const daySelect = day.find('select')
    await daySelect.setValue('12')
    const prefecture = wrapper.find('[data-test="prefecture-select-div"]')
    const prefectureSelect = prefecture.find('select')
    await prefectureSelect.setValue('東京都')
    const city = wrapper.find('[data-test="city-div"]')
    const cityInput = city.find('input')
    await cityInput.setValue('町田市')
    const addressLine1 = wrapper.find('[data-test="address-line1-div"]')
    const addressLine1Input = addressLine1.find('input')
    await addressLine1Input.setValue('森の里２−２２−２')
    const addressLine2 = wrapper.find('[data-test="address-line2-div"]')
    const addressLine2Input = addressLine2.find('input')
    await addressLine2Input.setValue('レオパレス２０３')
    const tel = wrapper.find('[data-test="tel-input-div"]')
    const telInput = tel.find('input')
    await telInput.setValue('09012345678')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessages = wrapper.findAllComponents(AlertMessage)
    expect(alertMessages.length).toBe(1)
    const alertMessage = alertMessages[0]
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.INVALID_MULTIPART_FORM_DATA_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_MULTIPART_FORM_DATA.toString())
  })
})
