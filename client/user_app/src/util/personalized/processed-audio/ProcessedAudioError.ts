export class ProcessedAudioError extends Error {
  constructor (message: string) {
    super(message)
    this.name = 'ProcessedAudioError'
    // instanceofを正しく動作させるためのワークアラウンド
    // https://blog.n-t.jp/post/tech/typescript-custom-error/
    Object.setPrototypeOf(this, ProcessedAudioError.prototype)
  }
}
