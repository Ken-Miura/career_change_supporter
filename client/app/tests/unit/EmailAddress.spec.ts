import { shallowMount } from '@vue/test-utils'
import EmailAddressInput from '@/components/EmailAddressInput.vue'

describe('EmailAddressInput.vue', () => {
  it('has one label and input', () => {
    const wrapper = shallowMount(EmailAddressInput)
    const labels = wrapper.findAll('label')
    expect(labels.length).toBe(1)
    const inputs = wrapper.findAll('input')
    expect(inputs.length).toBe(1)
  })

  it('emits on-email-address-updated event with input value', () => {
    const wrapper = shallowMount(EmailAddressInput)
    const input = wrapper.find('input')
    const pwd = 'test@example.com'
    input.setValue(pwd)
    const result = wrapper.emitted('on-email-address-updated')
    if (result === undefined || result[0] === undefined) {
      throw new Error('result === undefined || result[0] === undefined')
    }
    // 直前のチェックからresultとresult[0]がundefinedでないことは明白
    const resultArray = result[0] as string[]
    expect(resultArray[0]).toBe(pwd)
  })
})
