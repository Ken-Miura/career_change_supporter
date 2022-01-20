import { RouterLinkStub, mount, flushPromises } from '@vue/test-utils'
import ProfilePage from '@/views/personalized/ProfilePage.vue'
import { ref } from '@vue/runtime-dom'
import WaitingCircle from '@/components/WaitingCircle.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import { GetProfileResp } from '@/util/personalized/profile/GetProfileResp'
import { Identity } from '@/util/personalized/profile/Identity'
import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Code } from '@/util/Error'
import { Message } from '@/util/Message'
import TheHeader from '@/components/TheHeader.vue'
import { Career } from '@/util/personalized/profile/Career'

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

const getProfileDoneMock = ref(false)
const getProfileFuncMock = jest.fn()
jest.mock('@/util/personalized/profile/useGetProfile', () => ({
  useGetProfile: () => ({
    getProfileDone: getProfileDoneMock,
    getProfileFunc: getProfileFuncMock
  })
}))

describe('ProfilePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
    storeCommitMock.mockClear()
    identityMock = null
    getProfileDoneMock.value = false
    getProfileFuncMock.mockReset()
  })

  it('has WaitingCircle and TheHeader while api call finishes', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = false
    const wrapper = mount(ProfilePage, {
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

  it(`displays ${Message.UNEXPECTED_ERR} if unexpected error exists`, async () => {
    const apiErrResp = ApiErrorResp.create(500, ApiError.create(Code.UNEXPECTED_ERR_USER))
    getProfileFuncMock.mockResolvedValue(apiErrResp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
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
    expect(resultMessage).toContain(`${Message.UNEXPECTED_ERR} (${Code.UNEXPECTED_ERR_USER})`)
  })

  it(`displays alert message ${Message.UNEXPECTED_ERR} when connection error happened`, async () => {
    const errDetail = 'connection error'
    getProfileFuncMock.mockRejectedValue(new Error(errDetail))
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
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

  it(`moves to login if ${Code.UNAUTHORIZED} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(401, ApiError.create(Code.UNAUTHORIZED))
    getProfileFuncMock.mockResolvedValue(apiErrResp)
    getProfileDoneMock.value = true
    mount(ProfilePage, {
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

  it(`moves to terms of use if ${Code.NOT_TERMS_OF_USE_AGREED_YET} is returned`, async () => {
    const apiErrResp = ApiErrorResp.create(400, ApiError.create(Code.NOT_TERMS_OF_USE_AGREED_YET))
    getProfileFuncMock.mockResolvedValue(apiErrResp)
    getProfileDoneMock.value = true
    mount(ProfilePage, {
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

  it('has TheHeader after api call finishes', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const headers = wrapper.findAllComponents(TheHeader)
    expect(headers.length).toBe(1)
  })

  it('displays email address after api call finishes', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const emailAddressDiv = wrapper.find('[data-test="email-address"]')
    expect(emailAddressDiv.exists)
    const message = emailAddressDiv.text()
    expect(message).toContain('Eメールアドレス')
    expect(message).toContain('登録したEメールアドレスです。他のユーザーに公開されることはありません。')
    expect(message).toContain(`${profile.email_address}`)
  })

  it(', if no setting information found, displays that', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const noIdentitySetDiv = wrapper.find('[data-test="no-identity-set"]')
    expect(noIdentitySetDiv.exists)
    const noIdentitySetMessage = noIdentitySetDiv.text()
    expect(noIdentitySetMessage).toContain('ユーザー情報が設定されていません。')

    const noCareersSetDiv = wrapper.find('[data-test="no-careers-set"]')
    expect(noCareersSetDiv.exists)
    const noCareersSetMessage = noCareersSetDiv.text()
    expect(noCareersSetMessage).toContain('職務経歴は登録されていません。')

    const noFeePerHourInYerSetDiv = wrapper.find('[data-test="no-fee-per-hour-in-yen-set"]')
    expect(noFeePerHourInYerSetDiv.exists)
    const noFeePerHourInYerSetMessage = noFeePerHourInYerSetDiv.text()
    expect(noFeePerHourInYerSetMessage).toContain('相談料が設定されていません。')
  })

  it('displays identity information after api call finishes', async () => {
    const identity = {
    /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      sex: 'male' as 'male' | 'female',
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
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: identity,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const identitySetDiv = wrapper.find('[data-test="identity-set"]')
    expect(identitySetDiv.exists)
    const message = identitySetDiv.text()
    expect(message).toContain('名前')
    expect(message).toContain(`${identity.last_name} ${identity.first_name}`)
    expect(message).toContain('フリガナ')
    expect(message).toContain(`${identity.last_name_furigana} ${identity.first_name_furigana}`)
    expect(message).toContain('性別')
    // identity.sex === 'male' => 男
    expect(message).toContain('男')
    expect(message).toContain('生年月日')
    expect(message).toContain(`${identity.date_of_birth.year}年${identity.date_of_birth.month}月${identity.date_of_birth.day}日`)
    expect(message).toContain('住所')
    expect(message).toContain('都道府県')
    expect(message).toContain(`${identity.prefecture}`)
    expect(message).toContain('市区町村')
    expect(message).toContain(`${identity.city}`)
    expect(message).toContain('番地')
    expect(message).toContain(`${identity.address_line1}`)
    expect(message).toContain('建物名・部屋番号')
    expect(message).toContain(`${identity.address_line2}`)
    expect(message).toContain('電話番号')
    expect(message).toContain(`${identity.telephone_number}`)
  })

  it('displays 1 career information after api call finishes', async () => {
    const identity = {
    /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      sex: 'male' as 'male' | 'female',
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
    const career = {
      /* eslint-disable camelcase */
      id: 203,
      company_name: 'テスト株式会社',
      department_name: '営業部',
      office: '町田オフィス',
      career_start_date: {
        year: 2010,
        month: 4,
        day: 1
      },
      career_end_date: {
        year: 2016,
        month: 8,
        day: 1
      },
      contract_type: 'regular' as 'regular' | 'contract' | 'other',
      profession: '営業',
      annual_income_in_man_yen: 400,
      is_manager: false,
      position_name: null,
      is_new_graduate: true,
      note: 'テスト株式会社の営業での仕事内容や職場の雰囲気を教えることができます。'
      /* eslint-enable camelcase */
    }
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: identity,
      careers: [career],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const oneCereerDiv = wrapper.find('[data-test="careers-set"]')
    expect(oneCereerDiv.exists)
    const message = oneCereerDiv.text()
    expect(message).toContain('勤務先名称')
    expect(message).toContain(`${career.company_name}`)
    expect(message).toContain('雇用形態')
    // career.contract_type === 'regular' -> 正社員
    expect(message).toContain('正社員')
    expect(message).toContain('入社日')
    expect(message).toContain(`${career.career_start_date.year}年${career.career_start_date.month}月${career.career_start_date.day}日`)
    expect(message).toContain('退社日')
    expect(message).toContain(`${career.career_end_date.year}年${career.career_end_date.month}月${career.career_end_date.day}日`)
  })

  it('displays max num of careers information after api call finishes', async () => {
    const identity = {
    /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      sex: 'male' as 'male' | 'female',
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
    const careers = createMaxNumDummyCareers()
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: identity,
      careers,
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const oneCereerDiv = wrapper.find('[data-test="careers-set"]')
    expect(oneCereerDiv.exists)
    const message = oneCereerDiv.text()
    // 一つの職務経歴を表示したときにその他の表示を確認しているので、
    // ここでは最大数分会社名が表示されていることのみ確認する
    for (const career of careers) {
      expect(message).toContain(`${career.company_name}`)
    }
  })

  it('displays fee information after api call finishes', async () => {
    const identity = {
    /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      sex: 'male' as 'male' | 'female',
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
    const feePerHourInYen = 3000
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: identity,
      careers: [],
      fee_per_hour_in_yen: feePerHourInYen
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    await flushPromises()

    const feePerHourInYenDiv = wrapper.find('[data-test="fee-per-hour-in-yen-set"]')
    expect(feePerHourInYenDiv.exists)
    const message = feePerHourInYenDiv.text()
    expect(message).toContain(`${feePerHourInYen}円`)
  })

  it('moves to IdentityPage when "ユーザー情報を編集する" is pushed', async () => {
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: null,
      careers: [],
      fee_per_hour_in_yen: null
    /* eslint-enable camelcase */
    }
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-identity-page-button"]')
    expect(button.exists)
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('identity')
  })

  it('moves to AddCareerPage when "職務経歴を追加する" is pushed', async () => {
    const identity = {
      /* eslint-disable camelcase */
      last_name: '山田',
      first_name: '太郎',
      last_name_furigana: 'ヤマダ',
      first_name_furigana: 'タロウ',
      sex: 'male' as 'male' | 'female',
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
    const profile = {
      /* eslint-disable camelcase */
      email_address: 'test@test.com',
      identity: identity,
      careers: [],
      fee_per_hour_in_yen: null
      /* eslint-enable camelcase */
    }
    identityMock = identity
    const resp = GetProfileResp.create(profile)
    getProfileFuncMock.mockResolvedValue(resp)
    getProfileDoneMock.value = true
    const wrapper = mount(ProfilePage, {
      global: {
        stubs: {
          RouterLink: RouterLinkStub
        }
      }
    })
    const button = wrapper.find('[data-test="move-to-add-career-page-button"]')
    expect(button.exists)
    await button.trigger('click')

    expect(routerPushMock).toHaveBeenCalledTimes(1)
    expect(routerPushMock).toHaveBeenCalledWith('careers')
  })

  // TODO: ボタン押したときに画面遷移するテスト
  // TODO: ボタン押したときにユーザー情報がないとエラーメッセージを表示するテスト
})

function createMaxNumDummyCareers (): Career[] {
  const MAX_NUM_OF_CAREERS = 8
  const careers = []
  for (let i = 0; i < MAX_NUM_OF_CAREERS; i++) {
    let careerEndDate = null
    if (i !== (MAX_NUM_OF_CAREERS - 1)) {
      careerEndDate = {
        year: 2010 + (i + 1),
        month: 3,
        day: 31
      }
    }
    const career = {
      /* eslint-disable camelcase */
      id: i + 1,
      company_name: `テスト${i}株式会社`,
      department_name: '営業部',
      office: '町田オフィス',
      career_start_date: {
        year: 2010 + i,
        month: 4,
        day: 1
      },
      career_end_date: careerEndDate,
      contract_type: 'regular' as 'regular' | 'contract' | 'other',
      profession: '営業',
      annual_income_in_man_yen: 400,
      is_manager: false,
      position_name: null,
      is_new_graduate: (i === 0),
      note: `テスト${i}株式会社の営業での仕事内容や職場の雰囲気を教えることができます。`
      /* eslint-enable camelcase */
    }
    careers.push(career)
  }
  return careers
}
