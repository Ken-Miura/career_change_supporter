/* eslint-disable */
// @ts-ignore
export class WhiteNoiseProcessor extends AudioWorkletProcessor {
  // @ts-ignore
  process (inputs, outputs, parameters) {
    const output = outputs[0]
    // @ts-ignore
    output.forEach((channel) => {
      for (let i = 0; i < channel.length; i++) {
        channel[i] = Math.random() * 2 - 1
      }
    })
    return true
  }
}

// @ts-ignore
registerProcessor('white-noise-processor', WhiteNoiseProcessor)
