const recoveryCodeMock = null as string | null
jest.mock('vuex', () => ({
  useStore: () => ({
    state: {
      recoveryCode: recoveryCodeMock
    }
  })
}))

describe('EnableMfaConfirmationPage.vue', () => {
  beforeEach(() => {
    console.log('before')
  })

  it('test', async () => {
    console.log('test')
  })
})
