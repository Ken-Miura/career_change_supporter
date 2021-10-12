import { shallowMount } from '@vue/test-utils'
import AlertMessage from '@/components/AlertMessage.vue'

describe('AlertMessage.vue', () => {
  it('renders message passed', () => {
    const message = 'アラートメッセージ'
    const wrapper = shallowMount(AlertMessage, { props: { message } })
    expect(wrapper.text()).toMatch(message)
  })
})
