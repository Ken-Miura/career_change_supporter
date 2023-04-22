const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('DeleteAccountConfirmationPage.spec.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
  })

  it('has WaitingCircle while calling ', async () => {
    console.log('test')
  })
})
