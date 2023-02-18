export class SkyWayOriginatedError extends Error {
  constructor (message: string) {
    super(message)
    this.name = 'SkyWayOriginatedError'
    // instanceofを正しく動作させるためのワークアラウンド
    // https://blog.n-t.jp/post/tech/typescript-custom-error/
    Object.setPrototypeOf(this, SkyWayOriginatedError.prototype)
  }
}
