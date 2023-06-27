<template>
  <TheHeader/>
  <div class="bg-gradient-to-r from-gray-500 to-gray-900 min-h-screen pt-12 md:pt-20 pb-6 px-2 md:px-0" style="font-family:'Lato',sans-serif;">
    <div v-if="!requestsDone" class="m-6">
      <WaitingCircle />
    </div>
    <main v-else>
      <div v-if="outerErrorMessage" class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
        <AlertMessage v-bind:message="outerErrorMessage"/>
      </div>
      <div v-else>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="search-condition-label" class="font-bold text-2xl">検索条件</h3>
          <p data-test="search-condition-account-id" v-if="accountId" class="mt-4 ml-4 text-xl">アカウントID: {{ accountId }}</p>
          <p data-test="search-condition-email-address" v-else-if="emailAddress" class="mt-4 ml-4 text-xl">メールアドレス: {{ emailAddress }}</p>
          <p v-else class="mt-4 ml-4 text-xl">意図しない動作です。管理者に連絡して下さい</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 data-test="account-info-label" class="font-bold text-2xl">アカウント情報</h3>
          <div class="mt-4 ml-2">
            <div v-if="userAccount">
              <div class="m-4 text-2xl grid grid-cols-3">
                <div data-test="account-id" class="mt-2 justify-self-start col-span-1">アカウントID</div><div data-test="account-id-value" class="mt-2 justify-self-start col-span-2">{{ userAccount.user_account_id }}</div>
                <div data-test="email-address" class="mt-2 justify-self-start col-span-1">メールアドレス</div><div data-test="email-address-value" class="mt-2 justify-self-start col-span-2">{{ userAccount.email_address }}</div>
                <div data-test="created-at" class="mt-2 justify-self-start col-span-1">アカウント作成日</div><div data-test="created-at-value" class="mt-2 justify-self-start col-span-2">{{ userAccount.created_at }}</div>
                <div class="mt-2 justify-self-start col-span-1">最終ログイン日</div><div v-if="userAccount.last_login_time" class="mt-2 justify-self-start col-span-2">{{ userAccount.last_login_time }}</div><div v-else class="mt-2 justify-self-start col-span-2">未ログイン</div>
                <div class="mt-2 justify-self-start col-span-1">無効化日時</div><div v-if="userAccount.disabled_at" class="mt-2 justify-self-start col-span-2">{{ userAccount.disabled_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">無効化されていません</div>
                <div class="mt-2 justify-self-start col-span-1">二段階認証設定日</div><div v-if="userAccount.mfa_enabled_at" class="mt-2 justify-self-start col-span-2">{{ userAccount.mfa_enabled_at }}</div><div v-else class="mt-2 justify-self-start col-span-2">二段階認証は設定されていません</div>
              </div>
              <div class="mt-4 ml-2">
                <div class="text-2xl justify-self-start col-span-6 pt-3">
                  <p>アカウント無効化・有効化</p>
                </div>
                <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                  <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                    <div class="col-span-5">アカウントに対する操作が適正であることを確認しました</div>
                    <input v-model="accountEnableDisableConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                  </div>
                </div>
                <div v-if="userAccount.disabled_at">
                  <button v-on:click="enableAccount" v-bind:disabled="!accountEnableDisableConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">有効化する</button>
                </div>
                <div v-else>
                  <button v-on:click="disableAccount" v-bind:disabled="!accountEnableDisableConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">無効化する</button>
                </div>
                <div v-if="accountEnableDisableErrorMessage" class="mt-4">
                  <AlertMessage v-bind:message="accountEnableDisableErrorMessage"/>
                </div>
              </div>
              <div v-if="userAccount.mfa_enabled_at" class="mt-4 ml-2">
                <div class="text-2xl justify-self-start col-span-6 pt-3">
                  <p>二段階認証設定解除</p>
                </div>
                <div class="mt-2 min-w-full justify-self-start col-span-6 pt-2 rounded bg-gray-200">
                  <div class="p-4 text-xl grid grid-cols-6 justify-center items-center">
                    <div class="col-span-5">アカウントに対する操作が適正であることを確認しました</div>
                    <input v-model="disableMfaConfirmation" type="checkbox" class="ml-5 col-span-1 bg-gray-200 rounded h-6 w-6 text-gray-700 focus:outline-none border-b-4 border-gray-300 focus:border-gray-600 transition duration-500">
                  </div>
                </div>
                <button v-on:click="disableMfa" v-bind:disabled="!disableMfaConfirmation" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200 disabled:bg-slate-100 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none">二段階認証設定を解除する</button>
                <div v-if="disableMfaErrorMessage" class="mt-4">
                  <AlertMessage v-bind:message="disableMfaErrorMessage"/>
                </div>
              </div>
            </div>
            <div v-else>
              <p class="text-xl">アカウントが既に削除されている、または初めから存在しません。</p>
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">利用規約同意履歴</h3>
          <div v-if="!agreementsErrMessage">
            <div v-if="agreements.length !== 0">
              <ul>
                <li v-for="a in agreements" v-bind:key="a.version" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">利用規約バージョン {{ a.version }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">主体（メールアドレス）</div><div class="mt-2 justify-self-start col-span-2">{{ a.email_address }}</div>
                    <div class="mt-2 justify-self-start col-span-1">同意日時</div><div class="mt-2 justify-self-start col-span-2">{{ a.agreed_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              利用規約同意履歴は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="agreementsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">身分情報</h3>
          <div v-if="!identityErrMessage">
            <div v-if="identity" class="m-4 text-2xl grid grid-cols-3">
              <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ identity.last_name }} {{ identity.first_name }}</div>
              <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ identity.last_name_furigana }} {{ identity.first_name_furigana }}</div>
              <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ identity.date_of_birth.year }}年{{ identity.date_of_birth.month }}月{{ identity.date_of_birth.day }}日</div>
              <div class="mt-2 justify-self-start col-span-3">住所</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ identity.prefecture }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ identity.city }}</div>
              <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ identity.address_line1 }}</div>
              <div v-if="identity.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identity.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ identity.address_line2 }}</div>
              <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ identity.telephone_number }}</div>
            </div>
            <div v-else class="m-4 text-2xl">
              身分情報は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴</h3>
          <div v-if="!careersErrMessage">
            <div v-if="careers.length !== 0">
              <ul>
                <li v-for="c in careers" v-bind:key="c.career_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">職務経歴番号{{ c.career_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-2">{{ c.company_name }}</div>
                    <div v-if="c.department_name !== null" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="c.department_name !== null" class="mt-2 justify-self-start col-span-2">{{ c.department_name }}</div>
                    <div v-if="c.office !== null" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="c.office !== null" class="mt-2 justify-self-start col-span-2">{{ c.office }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="mt-2 justify-self-start col-span-2">{{ c.career_start_date }}</div>
                    <div v-if="c.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="c.career_end_date !== null" class="mt-2 justify-self-start col-span-2">{{ c.career_end_date }}</div>
                    <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
                    <div v-if="c.contract_type === 'regular'" class="mt-2 justify-self-start col-span-2">正社員</div>
                    <div v-else-if="c.contract_type === 'contract'" class="mt-2 justify-self-start col-span-2">契約社員</div>
                    <div v-else-if="c.contract_type === 'other'" class="mt-2 justify-self-start col-span-2">その他</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">想定外の値です。管理者にご連絡下さい</div>
                    <div v-if="c.profession !== null" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="c.profession !== null" class="mt-2 justify-self-start col-span-2">{{ c.profession }}</div>
                    <div v-if="c.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="c.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-2">{{ c.annual_income_in_man_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
                    <div v-if="c.is_manager" class="mt-2 justify-self-start col-span-2">管理職</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">非管理職</div>
                    <div v-if="c.position_name !== null" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="c.position_name !== null" class="mt-2 justify-self-start col-span-2">{{ c.position_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社区分</div>
                    <div v-if="c.is_new_graduate" class="mt-2 justify-self-start col-span-2">新卒入社</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">中途入社</div>
                    <div v-if="c.note !== null" class="mt-2 justify-self-start col-span-1">備考</div><div v-if="c.note !== null" class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ c.note }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              職務経歴は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="careersErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談一回（１時間）の相談料</h3>
          <div v-if="!feePerHourInYenErrMessage">
            <div v-if="feePerHourInYen" class="mt-6 ml-8 text-2xl">
              <p>{{ feePerHourInYen }}円</p>
            </div>
            <div v-else class="m-4 text-2xl">
              相談一回（１時間）の相談料は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="feePerHourInYenErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">テナント情報（報酬の入金口座、売上関連情報が紐づく識別子の情報）</h3>
          <div v-if="!tenantIdErrMessage">
            <div v-if="tenantId" class="mt-6 ml-8 text-2xl">
              <p>テナントID: {{ tenantId }}</p>
            </div>
            <div v-else class="m-4 text-2xl">
              テナント情報は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="tenantIdErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談申し込み</h3>
          <div v-if="!consultationReqsErrMessage">
            <div v-if="consultationReqs.length !== 0">
              <ul>
                <li v-for="consultationReq in consultationReqs" v-bind:key="consultationReq.consultation_req_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談依頼番号{{ consultationReq.consultation_req_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-7">
                    <div class="mt-2 justify-self-start col-span-3">相談申し込み先のアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.consultant_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第一候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.first_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第二候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.second_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第三候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.third_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（最遅の候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.latest_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.charge_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談料（円/時間）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.fee_per_hour_in_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.platform_fee_rate_in_percentage }}</div>
                    <div class="mt-2 justify-self-start col-span-3">与信枠開放日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultationReq.credit_facilities_expired_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              相談申し込みは見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationReqsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">相談受け付け</h3>
          <div v-if="!consultationOffersErrMessage">
            <div v-if="consultationOffers.length !== 0">
              <ul>
                <li v-for="consultationOffer in consultationOffers" v-bind:key="consultationOffer.consultation_req_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談依頼番号{{ consultationOffer.consultation_req_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-7">
                    <div class="mt-2 justify-self-start col-span-3">相談申し込み元のアカウントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.user_account_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第一候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.first_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第二候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.second_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（第三候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.third_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時（最遅の候補）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.latest_candidate_date_time }}</div>
                    <div class="mt-2 justify-self-start col-span-3">チャージID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.charge_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談料（円/時間）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.fee_per_hour_in_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-3">プラットフォーム利用手数料割合（%）</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.platform_fee_rate_in_percentage }}</div>
                    <div class="mt-2 justify-self-start col-span-3">与信枠開放日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultationOffer.credit_facilities_expired_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              相談受け付けは見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationOffersErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">ユーザーとしての相談一覧</h3>
          <div v-if="!consultationsAsUserErrMessage">
            <div v-if="consultationsAsUser.length !== 0">
              <ul>
                <li v-for="consultationAsUser in consultationsAsUser" v-bind:key="consultationAsUser.consultation_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談番号{{ consultationAsUser.consultation_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-7">
                    <div class="mt-2 justify-self-start col-span-3">コンサルタントID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.consultant_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.meeting_at }}</div>
                    <div class="mt-2 justify-self-start col-span-3">部屋名</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.room_name }}</div>
                    <div class="mt-2 justify-self-start col-span-3">ユーザー入室日時</div><div v-if="consultationAsUser.user_account_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 justify-self-start col-span-3">コンサルタント入室日時</div><div v-if="consultationAsUser.consultant_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsUser.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 w-full justify-self-start col-span-7">
                      <button v-on:click="moveToConsultationRelatedInfoPage(consultationAsUser.consultation_id)" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">決済、返金、評価状況を確認する</button>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              相談は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationsAsUserErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">コンサルタントとしての相談一覧</h3>
          <div v-if="!consultationsAsConsultantErrMessage">
            <div v-if="consultationsAsConsultant.length !== 0">
              <ul>
                <li v-for="consultationAsConsultant in consultationsAsConsultant" v-bind:key="consultationAsConsultant.consultation_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">相談番号{{ consultationAsConsultant.consultation_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-7">
                    <div class="mt-2 justify-self-start col-span-3">ユーザーID</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsConsultant.user_account_id }}</div>
                    <div class="mt-2 justify-self-start col-span-3">相談日時</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsConsultant.meeting_at }}</div>
                    <div class="mt-2 justify-self-start col-span-3">部屋名</div><div class="mt-2 justify-self-start col-span-4">{{ consultationAsConsultant.room_name }}</div>
                    <div class="mt-2 justify-self-start col-span-3">ユーザー入室日時</div><div v-if="consultationAsConsultant.user_account_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsConsultant.user_account_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 justify-self-start col-span-3">コンサルタント入室日時</div><div v-if="consultationAsConsultant.consultant_entered_at" class="mt-2 justify-self-start col-span-4">{{ consultationAsConsultant.consultant_entered_at }}</div><div v-else class="mt-2 justify-self-start col-span-4">入室記録なし</div>
                    <div class="mt-2 w-full justify-self-start col-span-7">
                      <button v-on:click="moveToConsultationRelatedInfoPage(consultationAsConsultant.consultation_id)" class="mt-4 min-w-full bg-gray-600 hover:bg-gray-700 text-white font-bold px-6 py-3 rounded shadow-lg hover:shadow-xl transition duration-200">決済、返金、評価状況を確認する</button>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              相談は見つかりませんでした
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="consultationsAsConsultantErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">ユーザーとしての評価</h3>
          <div v-if="!ratingInfoAsUserErrMessage">
            <div v-if="ratingInfoAsUser.average_rating" class="mt-6 ml-8 text-2xl">
              {{ ratingInfoAsUser.average_rating }}/5（評価件数：{{ ratingInfoAsUser.count }} 件）
            </div>
            <div v-else class="m-4 text-2xl">
              0/5（評価件数：{{ ratingInfoAsUser.count }} 件）
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="ratingInfoAsUserErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">コンサルタントとしての評価</h3>
          <div v-if="!ratingInfoAsConsultantErrMessage">
            <div v-if="ratingInfoAsConsultant.average_rating" class="mt-6 ml-8 text-2xl">
              {{ ratingInfoAsConsultant.average_rating }}/5（評価件数：{{ ratingInfoAsConsultant.count }} 件）
            </div>
            <div v-else class="m-4 text-2xl">
              0/5（評価件数：{{ ratingInfoAsConsultant.count }} 件）
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="ratingInfoAsConsultantErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認申請承認履歴（初回）</h3>
          <div v-if="!identityCreationApprovalRecordErrMessage">
            <div v-if="identityCreationApprovalRecord" class="mt-6 ml-8 text-2xl">
              <div class="m-4 text-2xl grid grid-cols-3">
                <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.last_name }} {{ identityCreationApprovalRecord.first_name }}</div>
                <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.last_name_furigana }} {{ identityCreationApprovalRecord.first_name_furigana }}</div>
                <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.date_of_birth }}</div>
                <div class="mt-2 justify-self-start col-span-3">住所</div>
                <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.prefecture }}</div>
                <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.city }}</div>
                <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.address_line1 }}</div>
                <div v-if="identityCreationApprovalRecord.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="identityCreationApprovalRecord.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.address_line2 }}</div>
                <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.telephone_number }}</div>
                <div class="mt-2 justify-self-start col-span-1">承認者</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.approved_by }}</div>
                <div class="mt-2 justify-self-start col-span-1">承認日時</div><div class="mt-2 justify-self-start col-span-2">{{ identityCreationApprovalRecord.approved_at }}</div>
              </div>
              <div class="m-2 text-2xl">
                <div class="mt-2">身分証明書画像（表面）</div>
                <img data-test="req-detail-image1" class="mt-2" v-bind:src="identityCreationApprovalRecord.image1_file_name_without_ext" />
              </div>
              <div v-if="identityCreationApprovalRecord.image2_file_name_without_ext" class="m-2 text-2xl">
                <div class="mt-2">身分証明書画像（裏面）</div>
                <img data-test="req-detail-image2" class="mt-2" v-bind:src="identityCreationApprovalRecord.image2_file_name_without_ext" />
              </div>
            </div>
            <div v-else class="m-4 text-2xl">
              本人確認申請承認履歴（初回）はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityCreationApprovalRecordErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認申請拒否履歴（初回）</h3>
          <div v-if="!identityCreationRejectionRecordsErrMessage">
            <div v-if="identityCreationRejectionRecords.length !== 0" class="mt-6 ml-8 text-2xl">
              <ul>
                <li v-for="rejectionRecord in identityCreationRejectionRecords" v-bind:key="rejectionRecord.rjd_cre_identity_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">本人確認申請拒否（初回）番号{{ rejectionRecord.rjd_cre_identity_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.last_name }} {{ rejectionRecord.first_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.last_name_furigana }} {{ rejectionRecord.first_name_furigana }}</div>
                    <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.date_of_birth }}</div>
                    <div class="mt-2 justify-self-start col-span-3">住所</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.prefecture }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.city }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.address_line1 }}</div>
                    <div v-if="rejectionRecord.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="rejectionRecord.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.address_line2 }}</div>
                    <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.telephone_number }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否理由</div><div class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ rejectionRecord.reason }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否者</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_by }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否日時</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              本人確認申請拒否履歴（初回）はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityCreationRejectionRecordsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認申請承認履歴（更新）</h3>
          <div v-if="!identityUpdateApprovalRecordsErrMessage">
            <div v-if="identityUpdateApprovalRecords.length !== 0" class="mt-6 ml-8 text-2xl">
              <ul>
                <li v-for="approvalRecord in identityUpdateApprovalRecords" v-bind:key="approvalRecord.appr_upd_identity_req_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">本人確認申請承認（更新）番号{{ approvalRecord.appr_upd_identity_req_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.last_name }} {{ approvalRecord.first_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.last_name_furigana }} {{ approvalRecord.first_name_furigana }}</div>
                    <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.date_of_birth }}</div>
                    <div class="mt-2 justify-self-start col-span-3">住所</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.prefecture }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.city }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.address_line1 }}</div>
                    <div v-if="approvalRecord.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="approvalRecord.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.address_line2 }}</div>
                    <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.telephone_number }}</div>
                    <div class="mt-2 justify-self-start col-span-1">承認者</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.approved_by }}</div>
                    <div class="mt-2 justify-self-start col-span-1">承認日時</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.approved_at }}</div>
                    <div class="m-2 col-span-3">
                      <div class="mt-2">身分証明書画像（表面）</div>
                      <img class="mt-2" v-bind:src="approvalRecord.image1_file_name_without_ext" />
                    </div>
                    <div v-if="approvalRecord.image2_file_name_without_ext" class="m-2 col-span-3">
                      <div class="mt-2">身分証明書画像（裏面）</div>
                      <img class="mt-2" v-bind:src="approvalRecord.image2_file_name_without_ext" />
                    </div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              本人確認申請承認履歴（更新）はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityUpdateApprovalRecordsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">本人確認申請拒否履歴（更新）</h3>
          <div v-if="!identityUpdateRejectionRecordsErrMessage">
            <div v-if="identityUpdateRejectionRecords.length !== 0" class="mt-6 ml-8 text-2xl">
              <ul>
                <li v-for="rejectionRecord in identityUpdateRejectionRecords" v-bind:key="rejectionRecord.rjd_upd_identity_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">本人確認申請拒否（更新）番号{{ rejectionRecord.rjd_upd_identity_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">氏名</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.last_name }} {{ rejectionRecord.first_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">フリガナ</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.last_name_furigana }} {{ rejectionRecord.first_name_furigana }}</div>
                    <div class="mt-2 justify-self-start col-span-1">生年月日</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.date_of_birth }}</div>
                    <div class="mt-2 justify-self-start col-span-3">住所</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">都道府県</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.prefecture }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">市区町村</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.city }}</div>
                    <div class="mt-2 ml-3 justify-self-start col-span-1">番地</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.address_line1 }}</div>
                    <div v-if="rejectionRecord.address_line2 !== null" class="mt-2 ml-3 justify-self-start col-span-1">建物名・部屋番号</div><div v-if="rejectionRecord.address_line2 !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.address_line2 }}</div>
                    <div class="mt-2 justify-self-start col-span-1">電話番号</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.telephone_number }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否理由</div><div class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ rejectionRecord.reason }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否者</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_by }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否日時</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              本人確認申請拒否履歴（更新）はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="identityUpdateRejectionRecordsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴確認申請承認履歴</h3>
          <div v-if="!careerCreationApprovalRecordsErrMessage">
            <div v-if="careerCreationApprovalRecords.length !== 0" class="mt-6 ml-8 text-2xl">
              <ul>
                <li v-for="approvalRecord in careerCreationApprovalRecords" v-bind:key="approvalRecord.appr_cre_career_req_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">職務経歴確認申請承認番号{{ approvalRecord.appr_cre_career_req_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.company_name }}</div>
                    <div v-if="approvalRecord.department_name !== null" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="approvalRecord.department_name !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.department_name }}</div>
                    <div v-if="approvalRecord.office !== null" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="approvalRecord.office !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.office }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.career_start_date }}</div>
                    <div v-if="approvalRecord.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="approvalRecord.career_end_date !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.career_end_date }}</div>
                    <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
                    <div v-if="approvalRecord.contract_type === 'regular'" class="mt-2 justify-self-start col-span-2">正社員</div>
                    <div v-else-if="approvalRecord.contract_type === 'contract'" class="mt-2 justify-self-start col-span-2">契約社員</div>
                    <div v-else-if="approvalRecord.contract_type === 'other'" class="mt-2 justify-self-start col-span-2">その他</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">想定外の値です。管理者にご連絡下さい</div>
                    <div v-if="approvalRecord.profession !== null" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="approvalRecord.profession !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.profession }}</div>
                    <div v-if="approvalRecord.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="approvalRecord.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.annual_income_in_man_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
                    <div v-if="approvalRecord.is_manager" class="mt-2 justify-self-start col-span-2">管理職</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">非管理職</div>
                    <div v-if="approvalRecord.position_name !== null" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="approvalRecord.position_name !== null" class="mt-2 justify-self-start col-span-2">{{ approvalRecord.position_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社区分</div>
                    <div v-if="approvalRecord.is_new_graduate" class="mt-2 justify-self-start col-span-2">新卒入社</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">中途入社</div>
                    <div v-if="approvalRecord.note !== null" class="mt-2 justify-self-start col-span-1">備考</div><div v-if="approvalRecord.note !== null" class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ approvalRecord.note }}</div>
                    <div class="mt-2 justify-self-start col-span-1">承認者</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.approved_by }}</div>
                    <div class="mt-2 justify-self-start col-span-1">承認日時</div><div class="mt-2 justify-self-start col-span-2">{{ approvalRecord.approved_at }}</div>
                    <div class="m-2 col-span-3">
                      <div class="mt-2">証明書類画像（表面）</div>
                      <img class="mt-2" v-bind:src="approvalRecord.image1_file_name_without_ext" />
                    </div>
                    <div v-if="approvalRecord.image2_file_name_without_ext" class="m-2 col-span-3">
                      <div class="mt-2">証明書類画像（裏面）</div>
                      <img class="mt-2" v-bind:src="approvalRecord.image2_file_name_without_ext" />
                    </div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              職務経歴確認申請承認履歴はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="careerCreationApprovalRecordsErrMessage"/>
          </div>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">職務経歴確認申請拒否履歴</h3>
          <div v-if="!careerCreationRejectionRecordsErrMessage">
            <div v-if="careerCreationRejectionRecords.length !== 0" class="mt-6 ml-8 text-2xl">
              <ul>
                <li v-for="rejectionRecord in careerCreationRejectionRecords" v-bind:key="rejectionRecord.rjd_cre_career_req_id" class="mt-4">
                  <div class="bg-gray-600 text-white font-bold rounded-t px-4 py-2">職務経歴確認申請拒否番号{{ rejectionRecord.rjd_cre_career_req_id }}</div>
                  <div class="border border-t-0 border-gray-600 rounded-b bg-white px-4 py-3 text-black text-xl grid grid-cols-3">
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.company_name }}</div>
                    <div v-if="rejectionRecord.department_name !== null" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="rejectionRecord.department_name !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.department_name }}</div>
                    <div v-if="rejectionRecord.office !== null" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="rejectionRecord.office !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.office }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.career_start_date }}</div>
                    <div v-if="rejectionRecord.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="rejectionRecord.career_end_date !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.career_end_date }}</div>
                    <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
                    <div v-if="rejectionRecord.contract_type === 'regular'" class="mt-2 justify-self-start col-span-2">正社員</div>
                    <div v-else-if="rejectionRecord.contract_type === 'contract'" class="mt-2 justify-self-start col-span-2">契約社員</div>
                    <div v-else-if="rejectionRecord.contract_type === 'other'" class="mt-2 justify-self-start col-span-2">その他</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">想定外の値です。管理者にご連絡下さい</div>
                    <div v-if="rejectionRecord.profession !== null" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="rejectionRecord.profession !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.profession }}</div>
                    <div v-if="rejectionRecord.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="rejectionRecord.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.annual_income_in_man_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
                    <div v-if="rejectionRecord.is_manager" class="mt-2 justify-self-start col-span-2">管理職</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">非管理職</div>
                    <div v-if="rejectionRecord.position_name !== null" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="rejectionRecord.position_name !== null" class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.position_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社区分</div>
                    <div v-if="rejectionRecord.is_new_graduate" class="mt-2 justify-self-start col-span-2">新卒入社</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">中途入社</div>
                    <div v-if="rejectionRecord.note !== null" class="mt-2 justify-self-start col-span-1">備考</div><div v-if="rejectionRecord.note !== null" class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ rejectionRecord.note }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否理由</div><div class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ rejectionRecord.reason }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否者</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_by }}</div>
                    <div class="mt-2 justify-self-start col-span-1">拒否日時</div><div class="mt-2 justify-self-start col-span-2">{{ rejectionRecord.rejected_at }}</div>
                  </div>
                </li>
              </ul>
            </div>
            <div v-else class="m-4 text-2xl">
              職務経歴確認申請拒否履歴はありません。
            </div>
          </div>
          <div v-else>
            <AlertMessage class="mt-4" v-bind:message="careerCreationRejectionRecordsErrMessage"/>
          </div>
        </div>
      </div>
    </main>
    <footer class="max-w-lg mx-auto flex flex-col text-white">
      <router-link to="/admin-menu" class="hover:underline text-center">管理メニューへ</router-link>
      <router-link to="/" class="mt-6 hover:underline text-center">トップページへ</router-link>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue'
import TheHeader from '@/components/TheHeader.vue'
import AlertMessage from '@/components/AlertMessage.vue'
import WaitingCircle from '@/components/WaitingCircle.vue'
import { useStore } from 'vuex'
import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { Message } from '@/util/Message'
import { usePostUserAccountRetrieval } from '@/util/personalized/user-account-info/usePostUserAccountRetrieval'
import { ApiErrorResp } from '@/util/ApiError'
import { UserAccountRetrievalResp } from '@/util/personalized/user-account-info/UserAccountRetrievalResp'
import { Code, createErrorMessage } from '@/util/Error'
import { useRouter } from 'vue-router'
import { UserAccount } from '@/util/personalized/user-account-info/UserAccount'
import { Identity } from '@/util/personalized/Identity'
import { useGetIdentityOptionByUserAccountId } from '@/util/personalized/user-account-info/identity/useGetIdentityOptionByUserAccountId'
import { GetIdentityOptionByUserAccountIdResp } from '@/util/personalized/user-account-info/identity/GetIdentityOptionByUserAccountIdResp'
import { useGetCareersByUserAccountId } from '@/util/personalized/user-account-info/career/useGetCareersByUserAccountId'
import { Career } from '@/util/personalized/user-account-info/career/Career'
import { GetCareersByUserAccountIdResp } from '@/util/personalized/user-account-info/career/GetCareersByUserAccountIdResp'
import { useGetFeePerHourInYenByUserAccountId } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/useGetFeePerHourInYenByUserAccountId'
import { GetFeePerHourInYenByUserAccountIdResp } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/GetFeePerHourInYenByUserAccountIdResp'
import { useGetTenantIdByUserAccountId } from '@/util/personalized/user-account-info/tenant/useGetTenantIdByUserAccountId'
import { GetTenantIdByUserAccountIdResp } from '@/util/personalized/user-account-info/tenant/GetTenantIdByUserAccountIdResp'
import { useGetAgreementsByUserAccountId } from '@/util/personalized/user-account-info/terms-of-use/useGetAgreementsByUserAccountId'
import { Agreement } from '@/util/personalized/user-account-info/terms-of-use/Agreement'
import { GetAgreementsByUserAccountIdResp } from '@/util/personalized/user-account-info/terms-of-use/GetAgreementsByUserAccountIdResp'
import { GetConsultationReqsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByUserAccountIdResp'
import { useGetConsultationReqsByUserAccountId } from '@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByUserAccountId'
import { ConsultationReq } from '@/util/personalized/user-account-info/consultation-req/ConsultationReq'
import { GetConsultationReqsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByConsultantIdResp'
import { useGetConsultationReqsByConsultantId } from '@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByConsultantId'
import { Consultation } from '@/util/personalized/Consultation'
import { GetConsultationsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByUserAccountIdResp'
import { useGetConsultationsByUserAccountId } from '@/util/personalized/user-account-info/consultation/useGetConsultationsByUserAccountId'
import { useGetConsultationsByConsultantId } from '@/util/personalized/user-account-info/consultation/useGetConsultationsByConsultantId'
import { GetConsultationsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByConsultantIdResp'
import { RatingInfoResult } from '@/util/personalized/user-account-info/rating-info/RatingInfoResult'
import { GetRatingInfoByUserAccountIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByUserAccountIdResp'
import { useGetRatingInfoByUserAccountId } from '@/util/personalized/user-account-info/rating-info/useGetRatingInfoByUserAccountId'
import { useGetRatingInfoByConsultantId } from '@/util/personalized/user-account-info/rating-info/useGetRatingInfoByConsultantId'
import { GetRatingInfoByConsultantIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByConsultantIdResp'
import { IdentityCreationApprovalRecord } from '@/util/personalized/user-account-info/identity-creation/IdentityCreationApprovalRecord'
import { useGetIdentityCreationApprovalRecord } from '@/util/personalized/user-account-info/identity-creation/useGetIdentityCreationApprovalRecord'
import { GetIdentityCreationApprovalRecordResp } from '@/util/personalized/user-account-info/identity-creation/GetIdentityCreationApprovalRecordResp'
import { IdentityCreationRejectionRecord } from '@/util/personalized/user-account-info/identity-creation/IdentityCreationRejectionRecord'
import { useGetIdentityCreationRejectionRecords } from '@/util/personalized/user-account-info/identity-creation/useGetIdentityCreationRejectionRecords'
import { GetIdentityCreationRejectionRecordsResp } from '@/util/personalized/user-account-info/identity-creation/GetIdentityCreationRejectionRecordsResp'
import { IdentityUpdateRejectionRecord } from '@/util/personalized/user-account-info/identity-update/IdentityUpdateRejectionRecord'
import { useGetIdentityUpdateRejectionRecords } from '@/util/personalized/user-account-info/identity-update/useGetIdentityUpdateRejectionRecords'
import { GetIdentityUpdateRejectionRecordsResp } from '@/util/personalized/user-account-info/identity-update/GetIdentityUpdateRejectionRecordsResp'
import { IdentityUpdateApprovalRecord } from '@/util/personalized/user-account-info/identity-update/IdentityUpdateApprovalRecord'
import { useGetIdentityUpdateApprovalRecords } from '@/util/personalized/user-account-info/identity-update/useGetIdentityUpdateApprovalRecords'
import { GetIdentityUpdateApprovalRecordsResp } from '@/util/personalized/user-account-info/identity-update/GetIdentityUpdateApprovalRecordsResp'
import { CareerCreationApprovalRecord } from '@/util/personalized/user-account-info/career-creation/CareerCreationApprovalRecord'
import { useGetCareerCreationApprovalRecords } from '@/util/personalized/user-account-info/career-creation/useGetCareerCreationApprovalRecords'
import { GetCareerCreationApprovalRecordsResp } from '@/util/personalized/user-account-info/career-creation/GetCareerCreationApprovalRecordsResp'
import { CareerCreationRejectionRecord } from '@/util/personalized/user-account-info/career-creation/CareerCreationRejectionRecord'
import { useGetCareerCreationRejectionRecords } from '@/util/personalized/user-account-info/career-creation/useGetCareerCreationRejectionRecords'
import { GetCareerCreationRejectionRecordsResp } from '@/util/personalized/user-account-info/career-creation/GetCareerCreationRejectionRecordsResp'
import { usePostDisableMfaReq } from '@/util/personalized/user-account-info/disable-mfa-req/usePostDisableMfaReq'
import { PostDisableMfaReqResp } from '@/util/personalized/user-account-info/disable-mfa-req/PostDisableMfaReqResp'
import { usePostDisableUserAccountReq } from '@/util/personalized/user-account-info/disable-user-account-req/usePostDisableUserAccountReq'
import { PostDisableUserAccountReqResp } from '@/util/personalized/user-account-info/disable-user-account-req/PostDisableUserAccountReqResp'
import { usePostEnableUserAccountReq } from '@/util/personalized/user-account-info/enable-user-account-req/usePostDisableUserAccountReq'
import { PostEnableUserAccountReqResp } from '@/util/personalized/user-account-info/enable-user-account-req/PostEnableUserAccountReqResp'

export default defineComponent({
  name: 'UserAccountInfoPage',
  components: {
    TheHeader,
    AlertMessage,
    WaitingCircle
  },
  setup () {
    const router = useRouter()
    const store = useStore()

    const accountId = ref(null as number | null)
    const emailAddress = ref(null as string | null)
    const outerErrorMessage = ref(null as string | null)

    const userAccount = ref(null as UserAccount | null)
    const {
      postUserAccountRetrievalDone,
      postUserAccountRetrievalByUserAccountIdFunc,
      postUserAccountRetrievalByEmailAddressFunc
    } = usePostUserAccountRetrieval()

    const getUserAccountByEitherAccountIdOrEmailAddress = async (accountId: number | null, emailAddress: string | null): Promise<UserAccountRetrievalResp | ApiErrorResp> => {
      if (accountId) {
        return postUserAccountRetrievalByUserAccountIdFunc(accountId)
      } else if (emailAddress) {
        return postUserAccountRetrievalByEmailAddressFunc(emailAddress)
      } else {
        throw new Error('Both accountId and emailAddress are null')
      }
    }

    const getUserAccount = async (accountId: number | null, emailAddress: string | null) => {
      try {
        const response = await getUserAccountByEitherAccountIdOrEmailAddress(accountId, emailAddress)
        if (!(response instanceof UserAccountRetrievalResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          outerErrorMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getResult()
        userAccount.value = result.user_account
      } catch (e) {
        outerErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const accountEnableDisableConfirmation = ref(false)
    const accountEnableDisableErrorMessage = ref(null as string | null)

    const {
      postDisableUserAccountReqDone,
      postDisableUserAccountReqFunc
    } = usePostDisableUserAccountReq()

    const disableAccount = async () => {
      const ua = userAccount.value
      if (!ua) {
        accountEnableDisableErrorMessage.value = `${Message.UNEXPECTED_ERR}: userAccount.value is null`
        return
      }
      try {
        const response = await postDisableUserAccountReqFunc(ua.user_account_id)
        if (!(response instanceof PostDisableUserAccountReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          accountEnableDisableErrorMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        accountEnableDisableErrorMessage.value = null
        const result = response.getUserAccountRetrievalResult()
        userAccount.value = result.user_account
      } catch (e) {
        accountEnableDisableErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        accountEnableDisableConfirmation.value = false
      }
    }

    const {
      postEnableUserAccountReqDone,
      postEnableUserAccountReqFunc
    } = usePostEnableUserAccountReq()

    const enableAccount = async () => {
      const ua = userAccount.value
      if (!ua) {
        accountEnableDisableErrorMessage.value = `${Message.UNEXPECTED_ERR}: userAccount.value is null`
        return
      }
      try {
        const response = await postEnableUserAccountReqFunc(ua.user_account_id)
        if (!(response instanceof PostEnableUserAccountReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          accountEnableDisableErrorMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        accountEnableDisableErrorMessage.value = null
        const result = response.getUserAccountRetrievalResult()
        userAccount.value = result.user_account
      } catch (e) {
        accountEnableDisableErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      } finally {
        accountEnableDisableConfirmation.value = false
      }
    }

    const disableMfaConfirmation = ref(false)
    const disableMfaErrorMessage = ref(null as string | null)

    const {
      postDisableMfaReqDone,
      postDisableMfaReqFunc
    } = usePostDisableMfaReq()

    const disableMfa = async () => {
      const ua = userAccount.value
      if (!ua) {
        disableMfaErrorMessage.value = `${Message.UNEXPECTED_ERR}: userAccount.value is null`
        return
      }
      try {
        const response = await postDisableMfaReqFunc(ua.user_account_id)
        if (!(response instanceof PostDisableMfaReqResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          disableMfaErrorMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        disableMfaErrorMessage.value = null
        const result = response.getUserAccountRetrievalResult()
        userAccount.value = result.user_account
      } catch (e) {
        disableMfaErrorMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const selectUserAccountId = (userAccount: UserAccount | null, userAccountId: number | null) => {
      if (userAccount) {
        return userAccount.user_account_id
      }
      if (userAccountId) {
        return userAccountId
      }
      return null
    }

    const agreements = ref([] as Agreement[])
    const {
      getAgreementsByUserAccountIdDone,
      getAgreementsByUserAccountIdFunc
    } = useGetAgreementsByUserAccountId()
    const agreementsErrMessage = ref(null as string | null)

    const findAgreements = async (accountId: number) => {
      try {
        const response = await getAgreementsByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetAgreementsByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          agreementsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getAgreementsResult()
        agreements.value = result.agreements
      } catch (e) {
        agreementsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const identity = ref(null as Identity | null)
    const {
      getIdentityOptionByUserAccountIdDone,
      getIdentityOptionByUserAccountIdFunc
    } = useGetIdentityOptionByUserAccountId()
    const identityErrMessage = ref(null as string | null)

    const findIdentity = async (accountId: number) => {
      try {
        const response = await getIdentityOptionByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetIdentityOptionByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          identityErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getIdentityResult()
        identity.value = result.identity_option
      } catch (e) {
        identityErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const careers = ref([] as Career[])
    const {
      getCareersByUserAccountIdDone,
      getCareersByUserAccountIdFunc
    } = useGetCareersByUserAccountId()
    const careersErrMessage = ref(null as string | null)

    const findCareers = async (accountId: number) => {
      try {
        const response = await getCareersByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetCareersByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          careersErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getCareersResult()
        careers.value = result.careers
      } catch (e) {
        careersErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const feePerHourInYen = ref(null as number | null)
    const {
      getFeePerHourInYenByUserAccountIdDone,
      getFeePerHourInYenByUserAccountIdFunc
    } = useGetFeePerHourInYenByUserAccountId()
    const feePerHourInYenErrMessage = ref(null as string | null)

    const findFeePerHourInYen = async (accountId: number) => {
      try {
        const response = await getFeePerHourInYenByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetFeePerHourInYenByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          feePerHourInYenErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getFeePerHourInYenResult()
        feePerHourInYen.value = result.fee_per_hour_in_yen
      } catch (e) {
        feePerHourInYenErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const tenantId = ref(null as string | null)
    const {
      getTenantIdByUserAccountIdDone,
      getTenantIdByUserAccountIdFunc
    } = useGetTenantIdByUserAccountId()
    const tenantIdErrMessage = ref(null as string | null)

    const findTenantId = async (accountId: number) => {
      try {
        const response = await getTenantIdByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetTenantIdByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          tenantIdErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getTenantIdResult()
        tenantId.value = result.tenant_id
      } catch (e) {
        tenantIdErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const consultationReqs = ref([] as ConsultationReq[])
    const {
      getConsultationReqsByUserAccountIdDone,
      getConsultationReqsByUserAccountIdFunc
    } = useGetConsultationReqsByUserAccountId()
    const consultationReqsErrMessage = ref(null as string | null)

    const findConsultationReqs = async (accountId: number) => {
      try {
        const response = await getConsultationReqsByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetConsultationReqsByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationReqsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationReqsResult()
        consultationReqs.value = result.consultation_reqs
      } catch (e) {
        consultationReqsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const consultationOffers = ref([] as ConsultationReq[])
    const {
      getConsultationReqsByConsultantIdDone,
      getConsultationReqsByConsultantIdFunc
    } = useGetConsultationReqsByConsultantId()
    const consultationOffersErrMessage = ref(null as string | null)

    const findConsultationOffers = async (accountId: number) => {
      try {
        const response = await getConsultationReqsByConsultantIdFunc(accountId.toString())
        if (!(response instanceof GetConsultationReqsByConsultantIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationOffersErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationReqsResult()
        consultationOffers.value = result.consultation_reqs
      } catch (e) {
        consultationOffersErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const consultationsAsUser = ref([] as Consultation[])
    const {
      getConsultationsByUserAccountIdDone,
      getConsultationsByUserAccountIdFunc
    } = useGetConsultationsByUserAccountId()
    const consultationsAsUserErrMessage = ref(null as string | null)

    const findConsultationsAsUser = async (accountId: number) => {
      try {
        const response = await getConsultationsByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetConsultationsByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationsAsUserErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationsResult()
        consultationsAsUser.value = result.consultations
      } catch (e) {
        consultationsAsUserErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const consultationsAsConsultant = ref([] as Consultation[])
    const {
      getConsultationsByConsultantIdDone,
      getConsultationsByConsultantIdFunc
    } = useGetConsultationsByConsultantId()
    const consultationsAsConsultantErrMessage = ref(null as string | null)

    const findConsultationsAsConsultant = async (accountId: number) => {
      try {
        const response = await getConsultationsByConsultantIdFunc(accountId.toString())
        if (!(response instanceof GetConsultationsByConsultantIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          consultationsAsConsultantErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getConsultationsResult()
        consultationsAsConsultant.value = result.consultations
      } catch (e) {
        consultationsAsConsultantErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const moveToConsultationRelatedInfoPage = async (consultationId: number) => {
      await router.push({ name: 'ConsultationRelatedInfoPage', params: { consultation_id: consultationId } })
    }

    const ratingInfoAsUser = ref({ average_rating: null, count: 0 } as RatingInfoResult)
    const {
      getRatingInfoByUserAccountIdDone,
      getRatingInfoByUserAccountIdFunc
    } = useGetRatingInfoByUserAccountId()
    const ratingInfoAsUserErrMessage = ref(null as string | null)

    const findRatingInfoAsUser = async (accountId: number) => {
      try {
        const response = await getRatingInfoByUserAccountIdFunc(accountId.toString())
        if (!(response instanceof GetRatingInfoByUserAccountIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          ratingInfoAsUserErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        ratingInfoAsUser.value = response.getRatingInfoResult()
      } catch (e) {
        ratingInfoAsUserErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const ratingInfoAsConsultant = ref({ average_rating: null, count: 0 } as RatingInfoResult)
    const {
      getRatingInfoByConsultantIdDone,
      getRatingInfoByConsultantIdFunc
    } = useGetRatingInfoByConsultantId()
    const ratingInfoAsConsultantErrMessage = ref(null as string | null)

    const findRatingInfoAsConsultant = async (accountId: number) => {
      try {
        const response = await getRatingInfoByConsultantIdFunc(accountId.toString())
        if (!(response instanceof GetRatingInfoByConsultantIdResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          ratingInfoAsConsultantErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        ratingInfoAsConsultant.value = response.getRatingInfoResult()
      } catch (e) {
        ratingInfoAsConsultantErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const identityCreationApprovalRecord = ref(null as IdentityCreationApprovalRecord | null)
    const {
      getIdentityCreationApprovalRecordDone,
      getIdentityCreationApprovalRecordFunc
    } = useGetIdentityCreationApprovalRecord()
    const identityCreationApprovalRecordErrMessage = ref(null as string | null)

    const findIdentityCreationApprovalRecord = async (accountId: number) => {
      try {
        const response = await getIdentityCreationApprovalRecordFunc(accountId.toString())
        if (!(response instanceof GetIdentityCreationApprovalRecordResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          identityCreationApprovalRecordErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getIdentityCreationApprovalRecordResult()
        const approvalRecord = result.approval_record
        if (approvalRecord) { // imgタグのv-bind:src内でそのまま指定して使えるように調整する
          approvalRecord.image1_file_name_without_ext = `/admin/api/identity-images/${approvalRecord.user_account_id}/${approvalRecord.image1_file_name_without_ext}`
          if (approvalRecord.image2_file_name_without_ext) {
            approvalRecord.image2_file_name_without_ext = `/admin/api/identity-images/${approvalRecord.user_account_id}/${approvalRecord.image2_file_name_without_ext}`
          }
        }
        identityCreationApprovalRecord.value = approvalRecord
      } catch (e) {
        identityCreationApprovalRecordErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const identityCreationRejectionRecords = ref([] as IdentityCreationRejectionRecord[])
    const {
      getIdentityCreationRejectionRecordsDone,
      getIdentityCreationRejectionRecordsFunc
    } = useGetIdentityCreationRejectionRecords()
    const identityCreationRejectionRecordsErrMessage = ref(null as string | null)

    const findIdentityCreationRejectionRecords = async (accountId: number) => {
      try {
        const response = await getIdentityCreationRejectionRecordsFunc(accountId.toString())
        if (!(response instanceof GetIdentityCreationRejectionRecordsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          identityCreationRejectionRecordsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getIdentityCreationRejectionRecordsResult()
        identityCreationRejectionRecords.value = result.rejection_records
      } catch (e) {
        identityCreationRejectionRecordsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const identityUpdateApprovalRecords = ref([] as IdentityUpdateApprovalRecord[])
    const {
      getIdentityUpdateApprovalRecordsDone,
      getIdentityUpdateApprovalRecordsFunc
    } = useGetIdentityUpdateApprovalRecords()
    const identityUpdateApprovalRecordsErrMessage = ref(null as string | null)

    const findIdentityUpdateApprovalRecords = async (accountId: number) => {
      try {
        const response = await getIdentityUpdateApprovalRecordsFunc(accountId.toString())
        if (!(response instanceof GetIdentityUpdateApprovalRecordsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          identityUpdateApprovalRecordsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getIdentityUpdateApprovalRecordsResult()
        const approvalRecords = result.approval_records
        for (let i = 0; i < approvalRecords.length; i++) { // imgタグのv-bind:src内でそのまま指定して使えるように調整する
          approvalRecords[i].image1_file_name_without_ext = `/admin/api/identity-images/${approvalRecords[i].user_account_id}/${approvalRecords[i].image1_file_name_without_ext}`
          if (approvalRecords[i].image2_file_name_without_ext) {
            approvalRecords[i].image2_file_name_without_ext = `/admin/api/identity-images/${approvalRecords[i].user_account_id}/${approvalRecords[i].image2_file_name_without_ext}`
          }
        }
        identityUpdateApprovalRecords.value = approvalRecords
      } catch (e) {
        identityUpdateApprovalRecordsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const identityUpdateRejectionRecords = ref([] as IdentityUpdateRejectionRecord[])
    const {
      getIdentityUpdateRejectionRecordsDone,
      getIdentityUpdateRejectionRecordsFunc
    } = useGetIdentityUpdateRejectionRecords()
    const identityUpdateRejectionRecordsErrMessage = ref(null as string | null)

    const findIdentityUpdateRejectionRecords = async (accountId: number) => {
      try {
        const response = await getIdentityUpdateRejectionRecordsFunc(accountId.toString())
        if (!(response instanceof GetIdentityUpdateRejectionRecordsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          identityUpdateRejectionRecordsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getIdentityUpdateRejectionRecordsResult()
        identityUpdateRejectionRecords.value = result.rejection_records
      } catch (e) {
        identityUpdateRejectionRecordsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const careerCreationApprovalRecords = ref([] as CareerCreationApprovalRecord[])
    const {
      getCareerCreationApprovalRecordsDone,
      getCareerCreationApprovalRecordsFunc
    } = useGetCareerCreationApprovalRecords()
    const careerCreationApprovalRecordsErrMessage = ref(null as string | null)

    const findCareerCreationApprovalRecords = async (accountId: number) => {
      try {
        const response = await getCareerCreationApprovalRecordsFunc(accountId.toString())
        if (!(response instanceof GetCareerCreationApprovalRecordsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          careerCreationApprovalRecordsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getCareerCreationApprovalRecordsResult()
        const approvalRecords = result.approval_records
        for (let i = 0; i < approvalRecords.length; i++) { // imgタグのv-bind:src内でそのまま指定して使えるように調整する
          approvalRecords[i].image1_file_name_without_ext = `/admin/api/career-images/${approvalRecords[i].user_account_id}/${approvalRecords[i].image1_file_name_without_ext}`
          if (approvalRecords[i].image2_file_name_without_ext) {
            approvalRecords[i].image2_file_name_without_ext = `/admin/api/career-images/${approvalRecords[i].user_account_id}/${approvalRecords[i].image2_file_name_without_ext}`
          }
        }
        careerCreationApprovalRecords.value = approvalRecords
      } catch (e) {
        careerCreationApprovalRecordsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    const careerCreationRejectionRecords = ref([] as CareerCreationRejectionRecord[])
    const {
      getCareerCreationRejectionRecordsDone,
      getCareerCreationRejectionRecordsFunc
    } = useGetCareerCreationRejectionRecords()
    const careerCreationRejectionRecordsErrMessage = ref(null as string | null)

    const findCareerCreationRejectionRecords = async (accountId: number) => {
      try {
        const response = await getCareerCreationRejectionRecordsFunc(accountId.toString())
        if (!(response instanceof GetCareerCreationRejectionRecordsResp)) {
          if (!(response instanceof ApiErrorResp)) {
            throw new Error(`unexpected result on getting request detail: ${response}`)
          }
          const code = response.getApiError().getCode()
          if (code === Code.UNAUTHORIZED) {
            await router.push('/login')
            return
          }
          careerCreationRejectionRecordsErrMessage.value = createErrorMessage(response.getApiError().getCode())
          return
        }
        const result = response.getCareerCreationRejectionRecordsResult()
        careerCreationRejectionRecords.value = result.rejection_records
      } catch (e) {
        careerCreationRejectionRecordsErrMessage.value = `${Message.UNEXPECTED_ERR}: ${e}`
      }
    }

    onMounted(async () => {
      const param = store.state.userAccountSearchParam as UserAccountSearchParam
      if (!param) {
        outerErrorMessage.value = Message.USER_ACCOUNT_SEARCH_PARAM_IS_NULL
        return
      }
      if (param.accountId === null && param.emailAddress === null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_EMPTY_MESSAGE
        return
      }
      if (param.accountId !== null && param.emailAddress !== null) {
        outerErrorMessage.value = Message.BOTH_ACCOUNT_ID_AND_EMAIL_ADDRESS_ARE_FILLED_MESSAGE
        return
      }
      accountId.value = param.accountId
      emailAddress.value = param.emailAddress

      await getUserAccount(param.accountId, param.emailAddress)

      const accId = selectUserAccountId(userAccount.value, param.accountId)
      if (!accId) {
        return
      }

      await findAgreements(accId)
      await findIdentity(accId)
      await findCareers(accId)
      await findFeePerHourInYen(accId)
      await findTenantId(accId)
      await findConsultationReqs(accId)
      await findConsultationOffers(accId)
      await findConsultationsAsUser(accId)
      await findConsultationsAsConsultant(accId)
      await findRatingInfoAsUser(accId)
      await findRatingInfoAsConsultant(accId)
      await findIdentityCreationApprovalRecord(accId)
      await findIdentityCreationRejectionRecords(accId)
      await findIdentityUpdateApprovalRecords(accId)
      await findIdentityUpdateRejectionRecords(accId)
      await findCareerCreationApprovalRecords(accId)
      await findCareerCreationRejectionRecords(accId)
    })

    const requestsDone = computed(() => {
      return (postUserAccountRetrievalDone.value &&
        getAgreementsByUserAccountIdDone.value &&
        getIdentityOptionByUserAccountIdDone.value &&
        getCareersByUserAccountIdDone.value &&
        getFeePerHourInYenByUserAccountIdDone.value &&
        getTenantIdByUserAccountIdDone.value &&
        getConsultationReqsByUserAccountIdDone.value &&
        getConsultationReqsByConsultantIdDone.value &&
        getConsultationsByUserAccountIdDone.value &&
        getConsultationsByConsultantIdDone.value &&
        getRatingInfoByUserAccountIdDone.value &&
        getRatingInfoByConsultantIdDone.value &&
        getIdentityCreationApprovalRecordDone.value &&
        getIdentityCreationRejectionRecordsDone.value &&
        getIdentityUpdateApprovalRecordsDone.value &&
        getIdentityUpdateRejectionRecordsDone.value &&
        getCareerCreationApprovalRecordsDone.value &&
        getCareerCreationRejectionRecordsDone.value &&
        postDisableUserAccountReqDone.value &&
        postEnableUserAccountReqDone.value &&
        postDisableMfaReqDone.value)
    })

    return {
      requestsDone,
      accountId,
      emailAddress,
      userAccount,
      accountEnableDisableConfirmation,
      accountEnableDisableErrorMessage,
      disableAccount,
      enableAccount,
      disableMfaConfirmation,
      disableMfaErrorMessage,
      disableMfa,
      agreements,
      agreementsErrMessage,
      identity,
      identityErrMessage,
      careers,
      careersErrMessage,
      feePerHourInYen,
      feePerHourInYenErrMessage,
      tenantId,
      tenantIdErrMessage,
      consultationReqs,
      consultationReqsErrMessage,
      consultationOffers,
      consultationOffersErrMessage,
      consultationsAsUser,
      consultationsAsUserErrMessage,
      consultationsAsConsultant,
      consultationsAsConsultantErrMessage,
      moveToConsultationRelatedInfoPage,
      ratingInfoAsUser,
      ratingInfoAsUserErrMessage,
      ratingInfoAsConsultant,
      ratingInfoAsConsultantErrMessage,
      identityCreationApprovalRecord,
      identityCreationApprovalRecordErrMessage,
      identityCreationRejectionRecords,
      identityCreationRejectionRecordsErrMessage,
      identityUpdateRejectionRecords,
      identityUpdateRejectionRecordsErrMessage,
      identityUpdateApprovalRecords,
      identityUpdateApprovalRecordsErrMessage,
      careerCreationApprovalRecords,
      careerCreationApprovalRecordsErrMessage,
      careerCreationRejectionRecords,
      careerCreationRejectionRecordsErrMessage,
      outerErrorMessage
    }
  }
})
</script>
