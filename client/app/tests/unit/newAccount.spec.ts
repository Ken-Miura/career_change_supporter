import { mount } from '@vue/test-utils'
import NewAccount from '@/views/NewAccount.vue'
import EmailAddress from '@/components/EmailAddress.vue'
import Password from '@/components/Password.vue'
import { useRouter } from 'vue-router'
import { createTempAccount } from '@/util/new-account/CreateTempAccount'
import { CreateTempAccountResp } from '@/util/new-account/CreateTempAccountResp'

jest.mock('@/util/new-account/CreateTempAccount')
const createTempAccountMock = createTempAccount as jest.MockedFunction<typeof createTempAccount>

jest.mock('vue-router')
const useRouterMock = useRouter as jest.MockedFunction<typeof useRouter>

describe('NewAccount.vue', () => {
  it('dispalays', async () => {
    const emailAddress = 'test@example.com'
    // const mockRoute = {
    //   params: {
    //     emailAddress: emailAddress
    //   }
    // }
    // const mockRouter = {
    //   push: jest.fn()
    // }
    createTempAccountMock.mockResolvedValue(CreateTempAccountResp.create(emailAddress))

    const wrapper = mount(NewAccount)
    // const wrapper = mount(NewAccount, {
    //   global: {
    //     mocks: {
    //       $route: mockRoute,
    //       $router: mockRouter
    //     }
    //   }
    // })

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
    
    // expect(mockRouter.push).toHaveBeenCalledTimes(1)
    // expect(mockRouter.push).toHaveBeenCalledWith('/temp-account-created')
  })
})
