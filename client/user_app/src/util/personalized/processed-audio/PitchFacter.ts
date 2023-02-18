import { Message } from '@/util/Message'
import { ProcessedAudioError } from './ProcessedAudioError'

// https://olvb.github.io/phaze/www/
// ではPitchに対して0.5から1.5までを使用している。
// アルゴリズムを理解できているわけではないので、素直に作者の作った値の範囲におさまるように使う。
// 1.0近辺はもとの声に近いため不使用。また0.5、1.5に近い値はかなり聞きづらい声になるため、それも避ける。
const LOWER_BOUNDARY_OF_LOW_PITCH_FACTOR = 0.6
const UPPER_BOUNDARY_OF_LOW_PITCH_FACTOR = 0.8
const LOWER_BOUNDARY_OF_HIGH_PITCH_FACTOR = 1.2
const UPPER_BOUNDARY_OF_HIGH_PITCH_FACTOR = 1.4

export function generatePitchFactor () {
  const num = getRandomInt(0, 2)
  if (num === 0) {
    return getRandomArbitrary(LOWER_BOUNDARY_OF_LOW_PITCH_FACTOR, UPPER_BOUNDARY_OF_LOW_PITCH_FACTOR)
  } else if (num === 1) {
    return getRandomArbitrary(LOWER_BOUNDARY_OF_HIGH_PITCH_FACTOR, UPPER_BOUNDARY_OF_HIGH_PITCH_FACTOR)
  } else {
    throw new ProcessedAudioError(`${Message.FAILED_TO_GENERATE_PITCH_FACTOR_MESSAGE} (num: ${num})`)
  }
}

/*
 * https://developer.mozilla.org/ja/docs/Web/JavaScript/Reference/Global_Objects/Math/random#2_%E3%81%A4%E3%81%AE%E5%80%A4%E3%81%AE%E9%96%93%E3%81%AE%E3%83%A9%E3%83%B3%E3%83%80%E3%83%A0%E3%81%AA%E6%95%B4%E6%95%B0%E3%82%92%E5%BE%97%E3%82%8B
 */
function getRandomInt (min: number, max: number) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min) + min) // The maximum is exclusive and the minimum is inclusive
}

/*
 * https://developer.mozilla.org/ja/docs/Web/JavaScript/Reference/Global_Objects/Math/random#2_%E3%81%A4%E3%81%AE%E5%80%A4%E3%81%AE%E9%96%93%E3%81%AE%E4%B9%B1%E6%95%B0%E3%82%92%E5%BE%97%E3%82%8B
 */
function getRandomArbitrary (min: number, max: number) {
  return Math.random() * (max - min) + min
}
