export class ApiError {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly code: number) {}
  public static create (code: number): ApiError {
    return new ApiError(code)
  }

  public getCode (): number {
    return this.code
  }
}

export class ApiErrorResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly statusCode: number, private readonly apiErr: ApiError) {}
  public static create (statusCode: number, apiErr: ApiError): ApiErrorResp {
    return new ApiErrorResp(statusCode, apiErr)
  }

  public getStatusCode (): number {
    return this.statusCode
  }

  public getApiError (): ApiError {
    return this.apiErr
  }
}
