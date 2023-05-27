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
          <h3 class="font-bold text-2xl">検索条件</h3>
          <p v-if="accountId" class="mt-4 ml-4 text-xl">アカウントID: {{ accountId }}</p>
          <p v-else-if="emailAddress" class="mt-4 ml-4 text-xl">メールアドレス: {{ emailAddress }}</p>
          <p v-else class="mt-4 ml-4 text-xl">意図しない動作です。管理者に連絡して下さい</p>
        </div>
        <div class="flex flex-col justify-center bg-white max-w-4xl mx-auto p-8 md:p-12 my-10 rounded-lg shadow-2xl">
          <h3 class="font-bold text-2xl">アカウント情報</h3>
          <div class="mt-4 ml-2">
            <div v-if="userAccount">
              <div class="m-4 text-2xl grid grid-cols-3">
                <div class="mt-2 justify-self-start col-span-1">アカウントID</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.user_account_id }}</div>
                <div class="mt-2 justify-self-start col-span-1">メールアドレス</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.email_address }}</div>
                <div class="mt-2 justify-self-start col-span-1">アカウント作成日</div><div class="mt-2 justify-self-start col-span-2">{{ userAccount.created_at }}</div>
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
                    <div class="mt-2 justify-self-start col-span-1">勤務先名称</div><div class="mt-2 justify-self-start col-span-2">{{ c.career.company_name }}</div>
                    <div v-if="c.career.department_name !== null" class="mt-2 justify-self-start col-span-1">部署名</div><div v-if="c.career.department_name !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.department_name }}</div>
                    <div v-if="c.career.office !== null" class="mt-2 justify-self-start col-span-1">勤務地</div><div v-if="c.career.office !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.office }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社日</div><div class="mt-2 justify-self-start col-span-2">{{ c.career.career_start_date.year }}年{{ c.career.career_start_date.month }}月{{ c.career.career_start_date.day }}日</div>
                    <div v-if="c.career.career_end_date !== null" class="mt-2 justify-self-start col-span-1">退社日</div><div v-if="c.career.career_end_date !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.career_end_date.year }}年{{ c.career.career_end_date.month }}月{{ c.career.career_end_date.day }}日</div>
                    <div class="mt-2 justify-self-start col-span-1">雇用形態</div>
                    <div v-if="c.career.contract_type === 'regular'" class="mt-2 justify-self-start col-span-2">正社員</div>
                    <div v-else-if="c.career.contract_type === 'contract'" class="mt-2 justify-self-start col-span-2">契約社員</div>
                    <div v-else-if="c.career.contract_type === 'other'" class="mt-2 justify-self-start col-span-2">その他</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">想定外の値です。管理者にご連絡下さい</div>
                    <div v-if="c.career.profession !== null" class="mt-2 justify-self-start col-span-1">職種</div><div v-if="c.career.profession !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.profession }}</div>
                    <div v-if="c.career.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-1">年収（単位：万円）</div><div v-if="c.career.annual_income_in_man_yen !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.annual_income_in_man_yen }}</div>
                    <div class="mt-2 justify-self-start col-span-1">管理職区分</div>
                    <div v-if="c.career.is_manager" class="mt-2 justify-self-start col-span-2">管理職</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">非管理職</div>
                    <div v-if="c.career.position_name !== null" class="mt-2 justify-self-start col-span-1">職位</div><div v-if="c.career.position_name !== null" class="mt-2 justify-self-start col-span-2">{{ c.career.position_name }}</div>
                    <div class="mt-2 justify-self-start col-span-1">入社区分</div>
                    <div v-if="c.career.is_new_graduate" class="mt-2 justify-self-start col-span-2">新卒入社</div>
                    <div v-else class="mt-2 justify-self-start col-span-2">中途入社</div>
                    <div v-if="c.career.note !== null" class="mt-2 justify-self-start col-span-1">備考</div><div v-if="c.career.note !== null" class="mt-2 justify-self-start col-span-2 whitespace-pre-wrap">{{ c.career.note }}</div>
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
import { CareersWithId } from '@/util/personalized/user-account-info/career/CareersWithId'
import { GetCareersByUserAccountIdResp } from '@/util/personalized/user-account-info/career/GetCareersByUserAccountIdResp'
import { useGetFeePerHourInYenByUserAccountId } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/useGetFeePerHourInYenByUserAccountId'
import { GetFeePerHourInYenByUserAccountIdResp } from '@/util/personalized/user-account-info/fee-per-hour-in-yen/GetFeePerHourInYenByUserAccountIdResp'
import { useGetTenantIdByUserAccountId } from '@/util/personalized/user-account-info/tenant/useGetTenantIdByUserAccountId'
import { GetTenantIdByUserAccountIdResp } from '@/util/personalized/user-account-info/tenant/GetTenantIdByUserAccountIdResp'
import { useGetAgreementsByUserAccountId } from '@/util/personalized/user-account-info/terms-of-use/useGetCareersByUserAccountId'
import { Agreement } from '@/util/personalized/user-account-info/terms-of-use/Agreement'
import { GetAgreementsByUserAccountIdResp } from '@/util/personalized/user-account-info/terms-of-use/GetAgreementsByUserAccountIdResp'
import { GetConsultationReqsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByUserAccountIdResp'
import { useGetConsultationReqsByUserAccountId } from '@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByUserAccountId'
import { ConsultationReq } from '@/util/personalized/user-account-info/consultation-req/ConsultationReq'
import { GetConsultationReqsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation-req/GetConsultationReqsByConsultantIdResp'
import { useGetConsultationReqsByConsultantId } from '@/util/personalized/user-account-info/consultation-req/useGetConsultationReqsByConsultantId'
import { Consultation } from '@/util/personalized/user-account-info/consultation/Consultation'
import { GetConsultationsByUserAccountIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByUserAccountIdResp'
import { useGetConsultationsByUserAccountId } from '@/util/personalized/user-account-info/consultation/useGetConsultationsByUserAccountId'
import { useGetConsultationsByConsultantId } from '@/util/personalized/user-account-info/consultation/useGetConsultationsByConsultantId'
import { GetConsultationsByConsultantIdResp } from '@/util/personalized/user-account-info/consultation/GetConsultationsByConsultantIdResp'
import { RatingInfoResult } from '@/util/personalized/user-account-info/rating-info/RatingInfoResult'
import { GetRatingInfoByUserAccountIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByUserAccountIdResp'
import { useGetRatingInfoByUserAccountId } from '@/util/personalized/user-account-info/rating-info/useGetRatingInfoByUserAccountId'
import { useGetRatingInfoByConsultantId } from '@/util/personalized/user-account-info/rating-info/useGetRatingInfoByConsultantId'
import { GetRatingInfoByConsultantIdResp } from '@/util/personalized/user-account-info/rating-info/GetRatingInfoByConsultantIdResp'

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
    }

    const accountEnableDisableConfirmation = ref(false)
    const accountEnableDisableErrorMessage = ref(null as string | null)
    const disableAccount = async () => {
      console.log('disableAccount') // 更新後のUserAccountを返してもらうようにする
    }
    const enableAccount = async () => {
      console.log('enableAccount') // 更新後のUserAccountを返してもらうようにする
    }

    const disableMfaConfirmation = ref(false)
    const disableMfaErrorMessage = ref(null as string | null)
    const disableMfa = async () => {
      console.log('disableMfa') // 更新後のUserAccountを返してもらうようにする
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
    }

    const identity = ref(null as Identity | null)
    const {
      getIdentityOptionByUserAccountIdDone,
      getIdentityOptionByUserAccountIdFunc
    } = useGetIdentityOptionByUserAccountId()
    const identityErrMessage = ref(null as string | null)

    const findIdentity = async (accountId: number) => {
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
    }

    const careers = ref([] as CareersWithId[])
    const {
      getCareersByUserAccountIdDone,
      getCareersByUserAccountIdFunc
    } = useGetCareersByUserAccountId()
    const careersErrMessage = ref(null as string | null)

    const findCareers = async (accountId: number) => {
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
    }

    const feePerHourInYen = ref(null as number | null)
    const {
      getFeePerHourInYenByUserAccountIdDone,
      getFeePerHourInYenByUserAccountIdFunc
    } = useGetFeePerHourInYenByUserAccountId()
    const feePerHourInYenErrMessage = ref(null as string | null)

    const findFeePerHourInYen = async (accountId: number) => {
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
    }

    const tenantId = ref(null as string | null)
    const {
      getTenantIdByUserAccountIdDone,
      getTenantIdByUserAccountIdFunc
    } = useGetTenantIdByUserAccountId()
    const tenantIdErrMessage = ref(null as string | null)

    const findTenantId = async (accountId: number) => {
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
    }

    const consultationReqs = ref([] as ConsultationReq[])
    const {
      getConsultationReqsByUserAccountIdDone,
      getConsultationReqsByUserAccountIdFunc
    } = useGetConsultationReqsByUserAccountId()
    const consultationReqsErrMessage = ref(null as string | null)

    const findConsultationReqs = async (accountId: number) => {
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
    }

    const consultationOffers = ref([] as ConsultationReq[])
    const {
      getConsultationReqsByConsultantIdDone,
      getConsultationReqsByConsultantIdFunc
    } = useGetConsultationReqsByConsultantId()
    const consultationOffersErrMessage = ref(null as string | null)

    const findConsultationOffers = async (accountId: number) => {
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
    }

    const consultationsAsUser = ref([] as Consultation[])
    const {
      getConsultationsByUserAccountIdDone,
      getConsultationsByUserAccountIdFunc
    } = useGetConsultationsByUserAccountId()
    const consultationsAsUserErrMessage = ref(null as string | null)

    const findConsultationsAsUser = async (accountId: number) => {
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
    }

    const consultationsAsConsultant = ref([] as Consultation[])
    const {
      getConsultationsByConsultantIdDone,
      getConsultationsByConsultantIdFunc
    } = useGetConsultationsByConsultantId()
    const consultationsAsConsultantErrMessage = ref(null as string | null)

    const findConsultationsAsConsultant = async (accountId: number) => {
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
    }

    const moveToConsultationRelatedInfoPage = async (consultationId: number) => {
      console.log(consultationId) // 決済、返金、評価状況を表示するページへ遷移する
    }

    const ratingInfoAsUser = ref({ average_rating: null, count: 0 } as RatingInfoResult)
    const {
      getRatingInfoByUserAccountIdDone,
      getRatingInfoByUserAccountIdFunc
    } = useGetRatingInfoByUserAccountId()
    const ratingInfoAsUserErrMessage = ref(null as string | null)

    const findInfoRatingAsUser = async (accountId: number) => {
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
    }

    const ratingInfoAsConsultant = ref({ average_rating: null, count: 0 } as RatingInfoResult)
    const {
      getRatingInfoByConsultantIdDone,
      getRatingInfoByConsultantIdFunc
    } = useGetRatingInfoByConsultantId()
    const ratingInfoAsConsultantErrMessage = ref(null as string | null)

    const findInfoRatingAsConsultant = async (accountId: number) => {
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
      await findInfoRatingAsUser(accId)
      await findInfoRatingAsConsultant(accId)
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
        getRatingInfoByConsultantIdDone.value)
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
      outerErrorMessage
    }
  }
})
</script>
