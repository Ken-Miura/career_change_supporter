import { shallowMount } from '@vue/test-utils'
import PassCodeInput from '@/components/PassCodeInput.vue'

describe('PassCodeInput.vue', () => {
  it('has one label and input', () => {
    const wrapper = shallowMount(PassCodeInput)
    const labels = wrapper.findAll('label')
    expect(labels.length).toBe(1)
    const inputs = wrapper.findAll('input')
    expect(inputs.length).toBe(1)
  })

  it('renders props.label when passed', () => {
    const label = 'パスコード'
    const wrapper = shallowMount(PassCodeInput, {
      props: { label }
    })
    expect(wrapper.text()).toMatch(label)
  })

  it('emits on-pass-code-updated event with input value', async () => {
    const wrapper = shallowMount(PassCodeInput)
    const input = wrapper.find('input')
    const passCode = '123456'
    await input.setValue(passCode)
    const result = wrapper.emitted('on-pass-code-updated')
    if (result === undefined || result[0] === undefined) {
      throw new Error('result === undefined || result[0] === undefined')
    }
    // 直前のチェックからresultとresult[0]がundefinedでないことは明白
    const resultArray = result[0] as string[]
    expect(resultArray[0]).toBe(passCode)
  })
})
