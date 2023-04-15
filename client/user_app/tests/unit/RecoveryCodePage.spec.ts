const routerPushMock = jest.fn()
jest.mock('vue-router', () => ({
  useRouter: () => ({
    push: routerPushMock
  })
}))

describe('RecoveryCodePage.vue', () => {
  beforeEach(() => {
    routerPushMock.mockClear()
  })

  it('test', async () => {
    console.log('test')
  })
})
