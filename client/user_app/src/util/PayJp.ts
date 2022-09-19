import { loadScript } from 'vue-plugin-load-script'

/// この関数は、アプリケーション中で一回だけしか呼び出してはいけない
// PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function createPayJp (): Promise<any> {
  const payJpJsUrl = 'https://js.pay.jp/v2/pay.js'
  try {
    await loadScript(payJpJsUrl)
  } catch (e) {
    throw new Error(`failed to load script from ${payJpJsUrl}: ${e}`)
  }
  const payJpPubKey = process.env.VUE_APP_PAYJP_PUBLIC_KEY
  // https://js.pay.jp/v2/pay.js 内でPayjpオブジェクトをwindowに追加している。
  // コンパイラは、ホストされたJavascriptファイル内で行われている処理を検知できないため、明示的にチェックを無視する。
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  const payjp = window.Payjp(payJpPubKey)
  return payjp
}
