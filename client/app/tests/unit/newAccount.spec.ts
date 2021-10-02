import { mount } from '@vue/test-utils'
import NewAccount from '@/views/NewAccount.vue'
import EmailAddress from '@/components/EmailAddress.vue'
import Password from '@/components/Password.vue'
import { createTempAccount } from '@/util/NewAccounts'
import { useRouter } from 'vue-router'

jest.mock('@/util/test')
const createTempAccountMock = createTempAccount as jest.MockedFunction<typeof createTempAccount>

jest.mock('vue-router')
const useRouterMock = useRouter as jest.MockedFunction<typeof useRouter>

describe('NewAccount.vue', () => {
  it('has one label and input', async () => {
    // createTempAccountMock.mockResolvedValue(200)

    const wrapper = mount(NewAccount)

    const emailAddr = wrapper.findComponent(EmailAddress)
    const emailAddrInput = emailAddr.find('input')
    emailAddrInput.setValue('test@example.com')

    const pwd = 'abcdABCD1234'
    const pwds = wrapper.findAllComponents(Password)
    const pwdInput = pwds[0].find('input')
    pwdInput.setValue(pwd)
    const pwdConfirmationInput = pwds[1].find('input')
    pwdConfirmationInput.setValue(pwd)

    const button = wrapper.find('button')
    await button.trigger('submit')
  })
})
