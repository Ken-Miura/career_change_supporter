import { shallowMount } from '@vue/test-utils'
import PasswordInput from '@/components/PasswordInput.vue'

describe('PasswordInput.vue', () => {
  it('has one label and input', () => {
    const wrapper = shallowMount(PasswordInput)
    const labels = wrapper.findAll('label')
    expect(labels.length).toBe(1)
    const inputs = wrapper.findAll('input')
    expect(inputs.length).toBe(1)
  })

  it('renders props.label when passed', () => {
    const label = 'パスワード'
    const wrapper = shallowMount(PasswordInput, {
      props: { label }
    })
    expect(wrapper.text()).toMatch(label)
  })

  it('emits on-password-updated event with input value', async () => {
    const wrapper = shallowMount(PasswordInput)
    const input = wrapper.find('input')
    const pwd = 'abcdABCD1234'
    await input.setValue(pwd)
    const result = wrapper.emitted('on-password-updated')
    if (result === undefined || result[0] === undefined) {
      throw new Error('result === undefined || result[0] === undefined')
    }
    // 直前のチェックからresultとresult[0]がundefinedでないことは明白
    const resultArray = result[0] as string[]
    expect(resultArray[0]).toBe(pwd)
  })
})
