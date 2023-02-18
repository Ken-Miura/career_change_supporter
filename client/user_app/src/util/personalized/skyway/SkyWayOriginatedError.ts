export class SkyWayOriginatedError extends Error {
  constructor (message: string) {
    super(message)
    this.name = 'SkyWayOriginatedError'
  }
}
