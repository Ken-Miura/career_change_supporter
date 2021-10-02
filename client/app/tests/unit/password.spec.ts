import { shallowMount } from '@vue/test-utils'
import Password from '@/components/Password.vue'

describe('Password.vue', () => {
  it('has one label and input', () => {
    const wrapper = shallowMount(Password)
    const labels = wrapper.findAll('label')
    expect(labels.length).toBe(1)
    const inputs = wrapper.findAll('input')
    expect(inputs.length).toBe(1)
  })

  it('renders props.label when passed', () => {
    const label = 'パスワード'
    const wrapper = shallowMount(Password, {
      props: { label }
    })
    expect(wrapper.text()).toMatch(label)
  })
})
