import { mount } from '@vue/test-utils'
import NewAccount from '@/views/NewAccount.vue'
import EmailAddress from '@/components/EmailAddress.vue'
import Password from '@/components/Password.vue'
import { createTempAccount } from '@/util/new-account/CreateTempAccount'
import { CreateTempAccountResp } from '@/util/new-account/CreateTempAccountResp'
import { useRouter } from 'vue-router'

jest.mock('@/util/new-account/CreateTempAccount')
const createTempAccountMock = createTempAccount as jest.MockedFunction<typeof createTempAccount>

// 参考: https://stackoverflow.com/questions/68763693/vue-routers-injection-fails-during-a-jest-unit-test
const routerPushMock = jest.fn();
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock,
  }),
}));
const routerMock = useRouter()

describe('NewAccount.vue', () => {
  it('dispalays', async () => {
    const emailAddress = 'test@example.com'
    createTempAccountMock.mockResolvedValue(CreateTempAccountResp.create(emailAddress))

    const wrapper = mount(NewAccount, {
      global: {
        // TODO: 要調査
        stubs: ['router-link']
      }
    })

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue(emailAddress)

    const pwd = 'abcdABCD1234'
    const pwds = wrapper.findAllComponents(Password)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(pwd)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(pwd)

    const button = wrapper.find('button')
    await button.trigger('submit')
    
    expect(routerPushMock).toHaveBeenCalledTimes(1)
    const data = JSON.parse(`{ "name": "TempAccountCreated", "params": {"emailAddress": "${emailAddress}"} }`)
    expect(routerPushMock).toHaveBeenCalledWith(data)
  })
})
