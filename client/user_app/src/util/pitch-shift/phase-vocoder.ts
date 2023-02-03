/* eslint-disable */
// @ts-nocheck
import { OLAProcessor } from './ola-processor.js'
import { FFT } from 'fft.js'

const BUFFERED_BLOCK_SIZE = 2048

function genHannWindow (length: number) {
  const win = new Float32Array(length)
  for (let i = 0; i < length; i++) {
    win[i] = 0.5 * (1 - Math.cos(2 * Math.PI * i / length))
  }
  return win
}

class PhaseVocoderProcessor extends OLAProcessor {
  static get parameterDescriptors () {
    return [{
      name: 'pitchFactor',
      defaultValue: 1.0
    }]
  }

  constructor (options) {
    options.processorOptions = {
      blockSize: BUFFERED_BLOCK_SIZE
    }
    super(options)

    this.fftSize = this.blockSize
    this.timeCursor = 0

    this.hannWindow = genHannWindow(this.blockSize)

    // prepare FFT and pre-allocate buffers
    this.fft = new FFT(this.fftSize)
    this.freqComplexBuffer = this.fft.createComplexArray()
    this.freqComplexBufferShifted = this.fft.createComplexArray()
    this.timeComplexBuffer = this.fft.createComplexArray()
    this.magnitudes = new Float32Array(this.fftSize / 2 + 1)
    this.peakIndexes = new Int32Array(this.magnitudes.length)
    this.nbPeaks = 0
  }

  processOLA (inputs, outputs, parameters) {
    // no automation, take last value
    const pitchFactor = parameters.pitchFactor[parameters.pitchFactor.length - 1]

    for (let i = 0; i < this.nbInputs; i++) {
      for (let j = 0; j < inputs[i].length; j++) {
        // big assumption here: output is symetric to input
        const input = inputs[i][j]
        const output = outputs[i][j]

        this.applyHannWindow(input)

        this.fft.realTransform(this.freqComplexBuffer, input)

        this.computeMagnitudes()
        this.findPeaks()
        this.shiftPeaks(pitchFactor)

        this.fft.completeSpectrum(this.freqComplexBufferShifted)
        this.fft.inverseTransform(this.timeComplexBuffer, this.freqComplexBufferShifted)
        this.fft.fromComplexArray(this.timeComplexBuffer, output)

        this.applyHannWindow(output)
      }
    }

    this.timeCursor += this.hopSize
  }

  /** Apply Hann window in-place */
  applyHannWindow (input) {
    for (let i = 0; i < this.blockSize; i++) {
      input[i] = input[i] * this.hannWindow[i]
    }
  }

  /** Compute squared magnitudes for peak finding **/
  computeMagnitudes () {
    let i = 0; let j = 0
    while (i < this.magnitudes.length) {
      const real = this.freqComplexBuffer[j]
      const imag = this.freqComplexBuffer[j + 1]
      // no need to sqrt for peak finding
      this.magnitudes[i] = real ** 2 + imag ** 2
      i += 1
      j += 2
    }
  }

  /** Find peaks in spectrum magnitudes **/
  findPeaks () {
    this.nbPeaks = 0
    let i = 2
    const end = this.magnitudes.length - 2

    while (i < end) {
      const mag = this.magnitudes[i]

      if (this.magnitudes[i - 1] >= mag || this.magnitudes[i - 2] >= mag) {
        i++
        continue
      }
      if (this.magnitudes[i + 1] >= mag || this.magnitudes[i + 2] >= mag) {
        i++
        continue
      }

      this.peakIndexes[this.nbPeaks] = i
      this.nbPeaks++
      i += 2
    }
  }

  /** Shift peaks and regions of influence by pitchFactor into new specturm */
  shiftPeaks (pitchFactor) {
    // zero-fill new spectrum
    this.freqComplexBufferShifted.fill(0)

    for (let i = 0; i < this.nbPeaks; i++) {
      const peakIndex = this.peakIndexes[i]
      const peakIndexShifted = Math.round(peakIndex * pitchFactor)

      if (peakIndexShifted > this.magnitudes.length) {
        break
      }

      // find region of influence
      let startIndex = 0
      let endIndex = this.fftSize
      if (i > 0) {
        const peakIndexBefore = this.peakIndexes[i - 1]
        startIndex = peakIndex - Math.floor((peakIndex - peakIndexBefore) / 2)
      }
      if (i < this.nbPeaks - 1) {
        const peakIndexAfter = this.peakIndexes[i + 1]
        endIndex = peakIndex + Math.ceil((peakIndexAfter - peakIndex) / 2)
      }

      // shift whole region of influence around peak to shifted peak
      const startOffset = startIndex - peakIndex
      const endOffset = endIndex - peakIndex
      for (let j = startOffset; j < endOffset; j++) {
        const binIndex = peakIndex + j
        const binIndexShifted = peakIndexShifted + j

        if (binIndexShifted >= this.magnitudes.length) {
          break
        }

        // apply phase correction
        const omegaDelta = 2 * Math.PI * (binIndexShifted - binIndex) / this.fftSize
        const phaseShiftReal = Math.cos(omegaDelta * this.timeCursor)
        const phaseShiftImag = Math.sin(omegaDelta * this.timeCursor)

        const indexReal = binIndex * 2
        const indexImag = indexReal + 1
        const valueReal = this.freqComplexBuffer[indexReal]
        const valueImag = this.freqComplexBuffer[indexImag]

        const valueShiftedReal = valueReal * phaseShiftReal - valueImag * phaseShiftImag
        const valueShiftedImag = valueReal * phaseShiftImag + valueImag * phaseShiftReal

        const indexShiftedReal = binIndexShifted * 2
        const indexShiftedImag = indexShiftedReal + 1
        this.freqComplexBufferShifted[indexShiftedReal] += valueShiftedReal
        this.freqComplexBufferShifted[indexShiftedImag] += valueShiftedImag
      }
    }
  }
}

registerProcessor('phase-vocoder-processor', PhaseVocoderProcessor)
