import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import AddCareerPage from '@/views/personalized/AddCareerPage.vue'
import { nextTick, reactive, ref } from 'vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { refresh } from '@/util/personalized/refresh/Refresh'
import TheHeader from '@/components/TheHeader.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { Message } from '@/util/Message'
import { RefreshResp } from '@/util/personalized/refresh/RefreshResp'
import { PostCareerResp } from '@/util/personalized/careers/PostCareerResp'
import { getMaxImageJpegImageSizeInBytes, MAX_JPEG_IMAGE_SIZE_IN_BYTES } from '@/util/MaxImageSize'

const waitingRequestDoneMock = ref(false)
const postCareerMock = jest.fn()
jest.mock('@/util/personalized/careers/usePostCareer', () => ({
  usePostCareer: () => ({
    waitingRequestDone: waitingRequestDoneMock,
    postCareerFunc: postCareerMock
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

// 画像ファイルのモックは下記を参考に行う
// https://stackoverflow.com/questions/24488985/how-to-mock-file-in-javascript

describe('AddCareerPage.vue', () => {
  beforeEach(() => {
    waitingRequestDoneMock.value = false
    postCareerMock.mockReset()
    refreshMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(MAX_JPEG_IMAGE_SIZE_IN_BYTES)
    onImage1StateChangeFuncMock.mockReset()
    onImage2StateChangeFuncMock.mockReset()
    routerPushMock.mockClear()
    imagesMock = reactive({
      image1: null as File | null,
      image2: null as File | null
    })
  })

  it('has one TheHeader, one submit button and one AlertMessage', () => {
    const wrapper = mount(AddCareerPage, {
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

  it('has labels and descriptions for career information', () => {
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const companyName = wrapper.find('[data-test="company-name-label"]')
    expect(companyName.exists)
    expect(companyName.text()).toContain('勤務先名称（必須）（例 xxx株式会社）')
    const departmentName = wrapper.find('[data-test="department-name-label"]')
    expect(departmentName.exists)
    expect(departmentName.text()).toContain('部署名（任意）')
    const office = wrapper.find('[data-test="office-label"]')
    expect(office.exists)
    expect(office.text()).toContain('勤務地（任意）（例 xxx事業所）')
    const careerStartDate = wrapper.find('[data-test="career-start-date-label"]')
    expect(careerStartDate.exists)
    expect(careerStartDate.text()).toContain('入社日（必須）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-label"]')
    expect(careerStarYear.exists)
    expect(careerStarYear.text()).toContain('年')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-label"]')
    expect(careerStarMonth.exists)
    expect(careerStarMonth.text()).toContain('月')
    const careerStarDay = wrapper.find('[data-test="career-start-day-label"]')
    expect(careerStarDay.exists)
    expect(careerStarDay.text()).toContain('日')
    const careerEndDate = wrapper.find('[data-test="career-end-date-label"]')
    expect(careerEndDate.exists)
    expect(careerEndDate.text()).toContain('退社日（任意）')
    const careerEndYear = wrapper.find('[data-test="career-end-year-label"]')
    expect(careerEndYear.exists)
    expect(careerEndYear.text()).toContain('年')
    const careerEndMonth = wrapper.find('[data-test="career-end-month-label"]')
    expect(careerEndMonth.exists)
    expect(careerEndMonth.text()).toContain('月')
    const careerEndDay = wrapper.find('[data-test="career-end-day-label"]')
    expect(careerEndDay.exists)
    expect(careerEndDay.text()).toContain('日')
    const contractType = wrapper.find('[data-test="contract-type-label"]')
    expect(contractType.exists)
    expect(contractType.text()).toContain('雇用形態（必須）')
    const profession = wrapper.find('[data-test="profession-label"]')
    expect(profession.exists)
    expect(profession.text()).toContain('職種（任意）（例 ITエンジニア）')
    const annualIncomInManYen = wrapper.find('[data-test="annual-incom-in-man-yen-label"]')
    expect(annualIncomInManYen.exists)
    expect(annualIncomInManYen.text()).toContain('年収（単位：万円）（任意）')
    const isManager = wrapper.find('[data-test="is-manager-label"]')
    expect(isManager.exists)
    expect(isManager.text()).toContain('管理職区分（必須）')
    const positionName = wrapper.find('[data-test="position-name-label"]')
    expect(positionName.exists)
    expect(positionName.text()).toContain('職位（任意）（例 係長）')
    const isNewGraduate = wrapper.find('[data-test="is-new-graduate-label"]')
    expect(isNewGraduate.exists)
    expect(isNewGraduate.text()).toContain('入社区分（必須）')
    const note = wrapper.find('[data-test="note-label"]')
    expect(note.exists)
    expect(note.text()).toContain('備考（任意）')
    const careerImageLabel = wrapper.find('[data-test="career-image-label"]')
    expect(careerImageLabel.exists)
    expect(careerImageLabel.text()).toContain('証明書類')
    const careerImageDescription = wrapper.find('[data-test="career-image-description"]')
    expect(careerImageDescription.exists)
    expect(careerImageDescription.text()).toContain('勤務先名称に記載した勤め先にご本人が勤務されていた証明として、書類をアップロードしていただきます。証明書類として、名刺、退職証明書または離職票をご利用になれます。証明書類の画像は、jpegかつサイズが4MB以下で、勤務先名称に記載した内容とご本人のお名前が記載されている必要があります。離職票をご利用の場合、マイナンバーが記載されていないこと（またはマイナンバーの箇所が隠されていること）を事前にご確認下さい。表面のアップロードは必須、裏面のアップロードは任意となります。')
    const careerImage1Label = wrapper.find('[data-test="career-image1-label"]')
    expect(careerImage1Label.exists)
    expect(careerImage1Label.text()).toContain('表面')
    const careerImage2Label = wrapper.find('[data-test="career-image2-label"]')
    expect(careerImage2Label.exists)
    expect(careerImage2Label.text()).toContain('裏面')
  })

  it('has AlertMessage with a hidden attribute when created', () => {
    const wrapper = mount(AddCareerPage, {
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
    waitingRequestDoneMock.value = true
    const wrapper = mount(AddCareerPage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned on opening AddCareerPage`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(AddCareerPage, {
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

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned on opening AddCareerPage`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    refreshMock.mockResolvedValue(apiErrResp)
    mount(AddCareerPage, {
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

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened on opening AddCareerPage`, async () => {
    const errDetail = 'connection error'
    refreshMock.mockRejectedValue(new Error(errDetail))
    const wrapper = mount(AddCareerPage, {
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

  it('moves to submit-career-success when postCareer is success (only mandatory input)', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image1.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: null as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size)
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-career-success')
  })

  it('moves to submit-career-success when postCareer is success (full input)', async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const departmentName = wrapper.find('[data-test="department-name-input"]').find('input')
    await departmentName.setValue('開発部　IOTソフトウェア開発')
    const office = wrapper.find('[data-test="office-input"]').find('input')
    await office.setValue('町田事業所')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('2000')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('8')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const careerEndYear = wrapper.find('[data-test="career-end-year-select"]').find('select')
    await careerEndYear.setValue('2008')
    const careerEndMonth = wrapper.find('[data-test="career-end-month-select"]').find('select')
    await careerEndMonth.setValue('7')
    const careerEndDay = wrapper.find('[data-test="career-end-day-select"]').find('select')
    await careerEndDay.setValue('31')
    const contractType = wrapper.find('[data-test="contract-type-select"]').find('select')
    await contractType.setValue('contract')
    const profession = wrapper.find('[data-test="profession-input"]').find('input')
    await profession.setValue('食品開発')
    const annualIncomeInManYen = wrapper.find('[data-test="annual-incom-in-man-yen-input"]').find('input')
    await annualIncomeInManYen.setValue('350')
    const isManager = wrapper.find('[data-test="is-manager-select"]').find('select')
    await isManager.setValue('true')
    const positionName = wrapper.find('[data-test="position-name-input"]').find('input')
    await positionName.setValue('課長')
    const isNewGraduate = wrapper.find('[data-test="is-new-graduate-select"]').find('select')
    await isNewGraduate.setValue('false')
    const note = wrapper.find('[data-test="note-input"]').find('input')
    await note.setValue(`備考は、
    改行を
    含むことが出来ます。`)

    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/submit-career-success')
  })

  it(`displays alert message ${Message.NO_CAREER_IMAGE1_SELECTED} when image1 is not selected`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image2 = new File(['test2'], 'image2.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: null as File | null,
      image2: image2 as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image2.size)
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.NO_CAREER_IMAGE1_SELECTED)
  })

  it(`displays alert message ${Message.NO_JPEG_EXTENSION_MESSAGE} when image1 file extension is not jpeg`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image.txt', { type: 'text/plain' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: null as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size)
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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

  it(`displays alert message ${Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE} when image1 exceeds max size`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
    // クライアントサイドでは拡張子とサイズしかチェックする予定はないので、実際のファイル形式と中身はなんでもよい
    const image1 = new File(['test'], 'image.jpeg', { type: 'image/jpeg' })
    imagesMock = reactive({
      image1: image1 as File | null,
      image2: null as File | null
    })
    getMaxImageJpegImageSizeInBytesMock.mockReset()
    getMaxImageJpegImageSizeInBytesMock.mockReturnValue(image1.size - 1)
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE)
  })

  it(`displays alert message ${Message.NO_JPEG_EXTENSION_MESSAGE} when image2 file extension is not jpeg`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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

  it(`displays alert message ${Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE} when image2 exceeds max size`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    postCareerMock.mockResolvedValue(PostCareerResp.create())
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(0)
    const alertMessage = wrapper.findComponent(AlertMessage)
    const classes = alertMessage.classes()
    expect(classes).not.toContain('hidden')
    const resultMessage = alertMessage.text()
    expect(resultMessage).toContain(Message.EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE)
  })

  it(`moves to login when ${Code.UNAUTHORIZED} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/login')
  })

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const submitButton = wrapper.find('[data-test="submit-button"]')
    await submitButton.trigger('submit')
    await nextTick()

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('/terms-of-use')
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const errDetail = 'connection error'
    postCareerMock.mockRejectedValue(new Error(errDetail))
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()
    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト（株）')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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

  it(`displays alert message ${Message.INVALID_COMPANY_NAME_LENGTH_MESSAGE} when company name length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_COMPANY_NAME_LENGTH))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.INVALID_COMPANY_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_COMPANY_NAME_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE} when company name has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_COMPANY_NAME))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('\u000A')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_COMPANY_NAME.toString())
  })

  it(`displays alert message ${Message.INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE} when department name length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_DEPARTMENT_NAME_LENGTH))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const departmentName = wrapper.find('[data-test="department-name-input"]').find('input')
    await departmentName.setValue('')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_DEPARTMENT_NAME_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE} when department name has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const departmentName = wrapper.find('[data-test="department-name-input"]').find('input')
    await departmentName.setValue('\u000D')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_DEPARTMENT_NAME.toString())
  })

  it(`displays alert message ${Message.INVALID_OFFICE_LENGTH_MESSAGE} when office length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_OFFICE_LENGTH))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const office = wrapper.find('[data-test="office-input"]').find('input')
    await office.setValue('')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.INVALID_OFFICE_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_OFFICE_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_OFFICE_MESSAGE} when office has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_OFFICE))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const office = wrapper.find('[data-test="office-input"]').find('input')
    await office.setValue('\u0009')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_OFFICE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_OFFICE.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CAREER_START_DATE_MESSAGE} when illegal career start date is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CAREER_START_DATE))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('32')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CAREER_START_DATE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CAREER_START_DATE.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CAREER_END_DATE_MESSAGE} when illegal career end date is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CAREER_END_DATE))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const careerEndYear = wrapper.find('[data-test="career-end-year-select"]').find('select')
    await careerEndYear.setValue('2000')
    const careerEndMonth = wrapper.find('[data-test="career-end-month-select"]').find('select')
    await careerEndMonth.setValue('5')
    const careerEndDay = wrapper.find('[data-test="career-end-day-select"]').find('select')
    await careerEndDay.setValue('32')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CAREER_END_DATE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CAREER_END_DATE.toString())
  })

  it(`displays alert message ${Message.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE_MESSAGE} when career start date exceeds career end date`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('2002')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('2')
    const careerEndYear = wrapper.find('[data-test="career-end-year-select"]').find('select')
    await careerEndYear.setValue('2002')
    const careerEndMonth = wrapper.find('[data-test="career-end-month-select"]').find('select')
    await careerEndMonth.setValue('4')
    const careerEndDay = wrapper.find('[data-test="career-end-day-select"]').find('select')
    await careerEndDay.setValue('1')
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
    expect(resultMessage).toContain(Message.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE_MESSAGE)
    expect(resultMessage).toContain(Code.CAREER_START_DATE_EXCEEDS_CAREER_END_DATE.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CONTRACT_TYPE_MESSAGE} when illegal contract type is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CONTRACT_TYPE))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('2002')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('2')
    const contractType = wrapper.find('[data-test="contract-type-select"]').find('select')
    await contractType.setValue('<script>alert(\'test\')</script>')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CONTRACT_TYPE_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CONTRACT_TYPE.toString())
  })

  it(`displays alert message ${Message.INVALID_PROFESSION_LENGTH_MESSAGE} when profession length is invalid`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_PROFESSION_LENGTH))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const profession = wrapper.find('[data-test="profession-input"]').find('input')
    await profession.setValue('')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.INVALID_PROFESSION_LENGTH_MESSAGE)
    expect(resultMessage).toContain(Code.INVALID_PROFESSION_LENGTH.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_CHAR_IN_PROFESSION_MESSAGE} when profession has illegal char`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_CHAR_IN_PROFESSION))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const profession = wrapper.find('[data-test="profession-input"]').find('input')
    await profession.setValue('\u001b')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
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
    expect(resultMessage).toContain(Message.ILLEGAL_CHAR_IN_PROFESSION_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_CHAR_IN_PROFESSION.toString())
  })

  it(`displays alert message ${Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE} when illegal annual income in man yen is passed`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const companyName = wrapper.find('[data-test="company-name-input"]').find('input')
    await companyName.setValue('テスト株式会社')
    const careerStarYear = wrapper.find('[data-test="career-start-year-select"]').find('select')
    await careerStarYear.setValue('1999')
    const careerStarMonth = wrapper.find('[data-test="career-start-month-select"]').find('select')
    await careerStarMonth.setValue('4')
    const careerStarDay = wrapper.find('[data-test="career-start-day-select"]').find('select')
    await careerStarDay.setValue('1')
    const annualIncomInManYen = wrapper.find('[data-test="annual-incom-in-man-yen-input"]').find('input')
    await annualIncomInManYen.setValue('100000')
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
    expect(resultMessage).toContain(Message.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE)
    expect(resultMessage).toContain(Code.ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN.toString())
  })

  it(`displays alert message ${Message.NO_NAME_FOUND_MESSAGE} (invalid request case 1)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NO_NAME_FOUND))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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

  it(`displays alert message ${Message.INVALID_MULTIPART_FORM_DATA_MESSAGE} (invalid request case 15)`, async () => {
    refreshMock.mockResolvedValue(RefreshResp.create())
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.INVALID_MULTIPART_FORM_DATA))
    postCareerMock.mockResolvedValue(apiErrResp)
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
    const wrapper = mount(AddCareerPage, {
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
