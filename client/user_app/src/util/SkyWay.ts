export function getSkyWayApiKey (): string {
  const skyWayApiKey = process.env.VUE_APP_SKY_WAY_API_KEY as string
  if (!skyWayApiKey) {
    throw new Error('No VUE_APP_SKY_WAY_API_KEY value found')
  }
  return skyWayApiKey
}
