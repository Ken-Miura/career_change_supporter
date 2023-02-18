export class ProcessedAudioError extends Error {
  constructor (message: string) {
    super(message)
    this.name = 'ProcessedAudioError'
  }
}
