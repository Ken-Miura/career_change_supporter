const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('EnableMfaConfirmationPage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
  })

  it('', async () => {
    console.log('test')
  })
})
