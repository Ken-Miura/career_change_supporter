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

  it('emits on-password-updated event with input value', () => {
    const wrapper = shallowMount(Password)
    const input = wrapper.find('input')
    const pwd = 'abcdABCD1234'
    input.setValue(pwd)
    const result = wrapper.emitted('on-password-updated')
    if (result === undefined || result[0] === undefined) {
      throw new Error('result === undefined || result[0] === undefined')
    }
    // 直前のチェックからresultとresult[0]がundefinedでないことは明白
    const resultArray = result[0] as string[]
    expect(resultArray[0]).toBe(pwd)
  })
})
