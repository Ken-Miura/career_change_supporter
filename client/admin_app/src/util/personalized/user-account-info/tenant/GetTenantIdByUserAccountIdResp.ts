import { TenantIdResult } from './TenantIdResult'

export class GetTenantIdByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly tenantIdResult: TenantIdResult) {}

  public static create (tenantIdResult: TenantIdResult): GetTenantIdByUserAccountIdResp {
    return new GetTenantIdByUserAccountIdResp(tenantIdResult)
  }

  public getTenantIdResult (): TenantIdResult {
    return this.tenantIdResult
  }
}
