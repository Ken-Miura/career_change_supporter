import { getMaxImageJpegImageSizeInBytes } from './MaxImageSize'

export function isJpegExtension (fileName: string): boolean {
  return fileName.endsWith('.jpg') || fileName.endsWith('.jpeg') || fileName.endsWith('.JPG') || fileName.endsWith('.JPEG') || fileName.endsWith('.jpe')
}

export function exceedJpegMaxImageSize (imageSizeInBytes: number): boolean {
  return exceedJpegMaxImageSizeInner(imageSizeInBytes, getMaxImageJpegImageSizeInBytes())
}

function exceedJpegMaxImageSizeInner (imageSizeInBytes: number, maxSizeInBytes: number): boolean {
  return imageSizeInBytes > maxSizeInBytes
}
