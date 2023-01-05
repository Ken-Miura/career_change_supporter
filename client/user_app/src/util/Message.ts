import { MAX_ANNUAL_INCOME_IN_MAN_YEN, MIN_ANNUAL_INCOME_IN_MAN_YEN } from './AnnualIncome'
import { MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN } from './Fee'

const RETRY_REQUEST = '通信環境を確認し、一定時間後に再度お試し下さい'

// classを利用を検討したが、constにできないためnamespaceを選択
// namespaceは、非推奨ではないため、代替可能な手段ができるまで利用
// eslint-disable-next-line
export namespace Message {
    export const PASSWORD_CONFIRMATION_FAILED = 'パスワードと確認用パスワードが一致していません'
    export const TEMP_ACCOUNT_CREATION_FAILED = `新規登録に失敗しました。${RETRY_REQUEST}`
    export const UNEXPECTED_ERR = `予期せぬエラーが発生しました。${RETRY_REQUEST}`
    export const INVALID_EMAIL_ADDRESS_FORMAT_MESSAGE = 'メールアドレスの形式が不正です'
    export const INVALID_PASSWORD_FORMAT_MESSAGE = 'パスワードの形式が不正です。英大文字、英小文字、数字、記号の内、2種類以上を組み合わせた10文字以上32文字以下の文字列を指定して下さい'
    export const ACCOUNT_ALREADY_EXISTS_MESSAGE = 'アカウントが既に存在しています'
    export const REACH_TEMP_ACCOUNTS_LIMIT_MESSAGE = '新規登録試行回数の上限に達しました。数日後、再度新規登録を行って下さい'
    export const INVALID_UUID_FORMAT_MESSAGE = 'UUIDの形式が不正です。URLが提供されたものと一致するかご確認下さい'
    export const TEMP_ACCOUNT_EXPIRED_MESSAGE = '有効期限が過ぎています。もう一度初めから手続きをお願い致します'
    export const NO_TEMP_ACCOUNT_FOUND_MESSAGE = '指定されたURLが見つかりませんでした。URLが提供されたものと一致するかご確認下さい'
    export const INVALID_QUERY_PARAM = '指定されたURLの形式が正しくありません。URLが提供されたものと一致するかご確認下さい'
    export const EMAIL_OR_PWD_INCORRECT_MESSAGE = 'メールアドレス、もしくはパスワードが間違っています'
    export const UNAUTHORIZED_MESSAGE = '認証が必要です'
    export const ACCOUNT_DISABLED_MESSAGE = 'アカウントが無効化されています'
    export const ACCOUNT_CREATED = '新規登録が完了しました。'
    export const ACCOUNT_CREATION_FAILED = `新規登録に失敗しました。${RETRY_REQUEST}`
    export const LOGIN_FAILED = `ログインに失敗しました。${RETRY_REQUEST}`
    export const PASSWORD_CHANGE_REQUEST_FAILED = `パスワード変更に失敗しました。${RETRY_REQUEST}`
    export const REACH_PASSWORD_CHANGE_REQ_LIMIT_MESSAGE = 'パスワード変更試行回数の上限に達しました。数日後、再度パスワード変更を行って下さい'
    export const PWD_CHANGE_REQ_EXPIRED_MESSAGE = TEMP_ACCOUNT_EXPIRED_MESSAGE
    export const PASSWORD_CHANGED_MESSAGE = 'パスワードを変更しました'
    export const NO_ACCOUNT_FOUND_MESSAGE = 'アカウントが存在しません'
    export const NO_PWD_CHANGE_REQ_FOUND_MESSAGE = NO_TEMP_ACCOUNT_FOUND_MESSAGE
    export const REACH_PAYMENT_PLATFORM_RATE_LIMIT_MESSAGE = '多量のリクエストを処理しているため、サーバーに負荷がかかっています。一定時間後に再度お試し下さい。'
    export const NO_IDENTITY_FOUND = 'ユーザー情報が設定されていません。先にユーザー情報を設定して下さい。'
    export const NO_IDENTITY_IMAGE1_SELECTED = '表面の画像が選択されていません。'
    export const INVALID_LAST_NAME_LENGTH_MESSAGE = '氏名の姓の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_LAST_NAME_MESSAGE = '氏名の姓に不正な文字が含まれています'
    export const INVALID_FIRST_NAME_LENGTH_MESSAGE = '氏名の名の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_FIRST_NAME_MESSAGE = '氏名の名に不正な文字が含まれています'
    export const INVALID_LAST_NAME_FURIGANA_LENGTH_MESSAGE = 'フリガナのセイの文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE = 'フリガナには全角カナのみをご利用下さい'
    export const INVALID_FIRST_NAME_FURIGANA_LENGTH_MESSAGE = 'フリガナのメイの文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA_MESSAGE = ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA_MESSAGE
    export const ILLEGAL_DATE_MESSAGE = '不正な生年月日です'
    export const ILLEGAL_AGE_MESSAGE = 'サービスをご利用可能な年齢に達していません'
    export const INVALID_PREFECTURE_MESSAGE = '不正な都道府県です'
    export const INVALID_CITY_LENGTH_MESSAGE = '住所の市区町村の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_CITY_MESSAGE = '住所の市区町村に不正な文字が含まれています'
    export const INVALID_ADDRESS_LINE1_LENGTH_MESSAGE = '住所の番地の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_ADDRESS_LINE1_MESSAGE = '住所の番地に不正な文字が含まれています'
    export const INVALID_ADDRESS_LINE2_LENGTH_MESSAGE = '住所の建物名・部屋番号の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_ADDRESS_LINE2_MESSAGE = '住所の建物名・部屋番号に不正な文字が含まれています'
    export const INVALID_TEL_NUM_FORMAT_MESSAGE = '不正な形式の電話番号です'
    export const NO_NAME_FOUND_MESSAGE = 'HTTPボディのmultipart/form-data内にnameが存在しないデータがあります'
    export const NO_FILE_NAME_FOUND_MESSAGE = 'HTTPボディのmultipart/form-data内で画像データに対してfilenameが存在していません'
    export const DATA_PARSE_FAILURE_MESSAGE = 'HTTPボディのmultipart/form-dataに不正なデータが含まれています'
    export const INVALID_NAME_IN_FIELD_MESSAGE = 'HTTPボディのmultipart/form-data内にnameに不正な文字列が含まれています'
    export const INVALID_UTF8_SEQUENCE_MESSAGE = 'UTF-8として不正な文字列です。エンコーディング方式にUTF-8を用いてください'
    export const INVALID_IDENTITY_JSON_MESSAGE = '不正な形式のユーザー情報です'
    export const NO_JPEG_EXTENSION_MESSAGE = '画像のファイル名がJPEGを示す拡張子ではありません。ファイル名は".jpg"、".jpeg"、".JPG"、".JPEG"または".jpe"で終わる必要があります'
    export const EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE = '画像のファイルサイズが4MBを超えています'
    export const INVALID_JPEG_IMAGE_MESSAGE = '身分証明書の画像がJPEG以外の形式です。身分証明書の画像にはJPEG形式をご利用下さい'
    export const NO_IDENTITY_FOUND_MESSAGE = 'ユーザー情報が含まれていない不正なリクエストです'
    export const NO_IDENTITY_IMAGE1_FOUND_MESSAGE = '身分証明書の表面の画像が含まれていない不正なリクエストです'
    export const IDENTITY_INFO_REQ_ALREADY_EXISTS_MESSAGE = '既に本人確認を依頼済みです。確認作業が終わるまでお待ち下さい。'
    export const SUBMIT_IDENTITY_SUCCESS_MESSAGE = '本人確認を受け付けました。本人確認作業の完了後、入力した値がユーザー情報に反映されます。'
    export const DATE_OF_BIRTH_IS_NOT_MATCH_MESSAGE = '生年月日を変更することはできません'
    export const NO_IDENTITY_UPDATED_MESSAGE = 'ユーザー情報に変更がありません'
    export const FIRST_NAME_IS_NOT_MATCH_MESSAGE = '氏名の内、名の変更はできません'
    export const INVALID_MULTIPART_FORM_DATA_MESSAGE = '不正なmultipart/form-dataです。他のブラウザで動作するかお試し下さい'
    export const NO_CAREER_IMAGE1_SELECTED = NO_IDENTITY_IMAGE1_SELECTED
    export const EXCEED_MAX_CAREER_IMAGE_SIZE_LIMIT_MESSAGE = EXCEED_MAX_IDENTITY_IMAGE_SIZE_LIMIT_MESSAGE
    export const NO_CAREER_START_DATE_INPUT = '入社日が入力されていません。'
    export const NO_PART_OF_CAREER_END_DATE_INPUT = '退社日の一部が入力されていません。'
    export const SUBMIT_CAREER_SUCCESS_MESSAGE = '職務経歴確認を受け付けました。確認作業の完了後、入力した値が職務経歴に反映されます。'
    export const INVALID_CAREER_JSON_MESSAGE = '不正な形式の職務経歴です'
    export const INVALID_COMPANY_NAME_LENGTH_MESSAGE = '勤務先名称の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_COMPANY_NAME_MESSAGE = '勤務先名称に不正な文字が含まれています（半角記号はご利用になれません。記号は全角記号でご入力下さい）'
    export const INVALID_DEPARTMENT_NAME_LENGTH_MESSAGE = '部署名の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_DEPARTMENT_NAME_MESSAGE = '部署名に不正な文字が含まれています（半角記号はご利用になれません。記号は全角記号でご入力下さい）'
    export const INVALID_OFFICE_LENGTH_MESSAGE = '勤務地の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_OFFICE_MESSAGE = '勤務地に不正な文字が含まれています（半角記号と空白はご利用になれません。記号は全角記号でご入力下さい）'
    export const ILLEGAL_CAREER_START_DATE_MESSAGE = '不正な入社日が指定されています'
    export const ILLEGAL_CAREER_END_DATE_MESSAGE = '不正な退社日が指定されています'
    export const CAREER_START_DATE_EXCEEDS_CAREER_END_DATE_MESSAGE = '入社日が退社日を超えて指定されています'
    export const ILLEGAL_CONTRACT_TYPE_MESSAGE = '不正な雇用形態が指定されています'
    export const INVALID_PROFESSION_LENGTH_MESSAGE = '職種の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_PROFESSION_MESSAGE = '職種に不正な文字が含まれています（半角記号と空白はご利用になれません。記号は全角記号でご入力下さい）'
    export const ILLEGAL_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE = `年収（単位：万円）には${MIN_ANNUAL_INCOME_IN_MAN_YEN}以上${MAX_ANNUAL_INCOME_IN_MAN_YEN}以下の数字を指定して下さい`
    export const INVALID_POSITION_NAME_LENGTH_MESSAGE = '職位の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_POSITION_NAME_MESSAGE = '職位に不正な文字が含まれています（半角記号と空白はご利用になれません。記号は全角記号でご入力下さい）'
    export const INVALID_NOTE_LENGTH_MESSAGE = '備考の文字数が規定範囲外です'
    export const ILLEGAL_CHAR_IN_NOTE_MESSAGE = '備考に不正な文字が含まれています（半角記号はご利用になれません。記号は全角記号でご入力下さい）'
    export const NO_CAREER_FOUND_MESSAGE = '職務経歴が含まれていない不正なリクエストです'
    export const NO_CAREER_IMAGE1_FOUND_MESSAGE = '証明書類の表面の画像が含まれていない不正なリクエストです'
    export const REACH_CAREER_NUM_LIMIT_MESSAGE = '既に最大数の職務経歴が登録されています'
    export const NO_IDENTITY_REGISTERED_MESSAGE = 'ユーザー情報が登録されていません。先にユーザー情報を登録して下さい'
    export const REACH_CREATE_CAREER_REQ_NUM_LIMIT_MESSAGE = '既に複数の職務経歴の確認依頼を行っています。確認作業が完了するまでお待ち下さい'
    export const DELETE_CAREER_SUCCESS_MESSAGE = '職務経歴を削除しました。'
    export const NO_CAREER_TO_HANDLE_FOUND_MESSAGE = '職務経歴が存在しません'
    export const ILLEGAL_FEE_PER_HOUR_IN_YEN_MESSAGE = `相談料は${MIN_FEE_PER_HOUR_IN_YEN}円以上、${MAX_FEE_PER_HOUR_IN_YEN}円以下の値を設定して下さい`
    export const SUBMIT_FEE_PER_HOUR_IN_YEN_SUCCESS_MESSAGE = '相談料を設定しました。'
    export const SUBMIT_BANK_ACCOUNT_SUCCESS_MESSAGE = '報酬の入金口座を設定しました。'
    export const INVALID_BANK_CODE_FORMAT_MESSAGE = '不正な形式の銀行コードです'
    export const INVALID_BRANCH_CODE_FORMAT_MESSAGE = '不正な形式の支店コードです'
    export const INVALID_ACCOUNT_TYPE_MESSAGE = '不正な形式の預金種別です'
    export const INVALID_ACCOUNT_NUMBER_FORMAT_MESSAGE = '不正な形式の口座番号です'
    export const INVALID_ACCOUNT_HOLDER_NAME_LENGTH_MESSAGE = '不正な長さの口座名義です'
    export const ILLEGAL_CHAR_IN_ACCOUNT_HOLDER_NAME_MESSAGE = '不正な形式の口座名義です'
    export const ACCOUNT_HOLDER_NAME_DOES_NOT_MATCH_FULL_NAME_MESSAGE = '口座名義とユーザー情報で本人確認された情報と異なります。セイ、メイの間に全角空白１文字が入っていること（全角空白１文字以外に余計な文字が含まれていないこと）をご確認下さい'
    export const INVALID_BANK_MESSAGE = '不正な銀行コードです。正しい銀行コードか再度お確かめの上、ご入力下さい'
    export const INVALID_BANK_BRANCH_MESSAGE = '不正な支店コードです。正しい支店コードか再度お確かめの上、ご入力下さい'
    export const INVALID_BANK_ACCOUNT_NUMBER_MESSAGE = '不正な口座番号です。口座番号の桁数が適切か再度お確かめの上、ご入力下さい'
    export const ILLEGAL_YEARS_OF_SERVICE_MESSAGE = '不正な在籍年数です'
    export const EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_ANNUAL_INCOME_IN_MAN_YEN_MESSAGE = '年収に関して、以上で指定した年収が以下で指定した年収を超えています'
    export const EQUAL_OR_MORE_EXCEEDS_EQUAL_OR_LESS_IN_FEE_PER_HOUR_IN_YEN_MESSAGE = '相談一回（１時間）の相談料に関して、以上で指定した金額が以下で指定した金額を超えています'
    export const INVALID_SORT_KEY_MESSAGE = '不正なソートキーです'
    export const INVALID_SORT_ORDER_MESSAGE = '不正なソートオーダーです'
    export const INVALID_CONSULTANT_SEARCH_PARAM_FROM_MESSAGE = '不正な検索結果の始点が指定されています'
    export const INVALID_CONSULTANT_SEARCH_PARAM_SIZE_MESSAGE = '不正なページサイズです'
    export const NO_CONSULTANT_SEARCH_PARAM_FOUND_MESSAGE = '相談申し込みから相談相手の検索を行って下さい'
    export const EQUAL_OR_MORE_IS_LESS_THAN_OR_MORE_YEARS_OF_SERVICE_MESSAGE = '在籍年数に関して、"年以上"または"年未満"の指定が不正です（"年以上"は"年未満"よりも小さい値である必要が有ります）'
    export const NO_CAREERS_FOUND_MESSAGE = '職務経歴が登録されていません。報酬の入金口座の登録前に職務経歴を登録して下さい'
    export const NO_FEE_PER_HOUR_IN_YEN_FOUND_MESSAGE = '相談一回（１時間）の相談料が登録されていません。報酬の入金口座の登録前に相談一回（１時間）の相談料を登録して下さい'
    export const NON_POSITIVE_CONSULTANT_ID_MESSAGE = '不正なコンサルタントIDです'
    export const CONSULTANT_DOES_NOT_EXIST_MESSAGE = 'コンサルタントが見つかりません'
    export const FEE_PER_HOUR_IN_YEN_WAS_UPDATED_MESSAGE = 'コンサルタントの相談料が更新されています'
    export const CONSULTANT_IS_NOT_AVAILABLE_MESSAGE = `${CONSULTANT_DOES_NOT_EXIST_MESSAGE}`
    export const PROFIT_OBJECTIVE_USE_IS_NOT_ALLOWED_MESSAGE = '事業者、個人事業主による営利目的の利用はできません'
    export const UNAUTHORIZED_ON_CARD_OPERATION_MESSAGE = 'ログインが必要です。決済処理は行われていないため、再度ログインし、相談の申し込みを行って下さい'
    export const NOT_TERMS_OF_USE_AGREED_YET_ON_CARD_OPERATION_MESSAGE = '利用規約への同意が必要です。決済処理は行われていないため、再度ログインし、利用規約への同意後、相談の申し込みを行って下さい'
    export const ILLEGAL_CONSULTATION_DATE_TIME_MESSAGE = '不正な相談開始日時です'
    export const ILLEGAL_CONSULTATION_HOUR_MESSAGE = '不正な相談開始時間です'
    export const INVALID_CONSULTATION_DATE_TIME_MESSAGE = '申し込み可能な相談開始日時の範囲外です'
    export const DUPLICATE_DATE_TIME_CANDIDATES_MESSAGE = '相談開始日時が重複しています'
    export const THREE_D_SECURE_ERROR_MESSAGE = 'クレジットカードの処理時にエラーが発生しました。アカウントのメールアドレス、エラー発生時刻を記載し、問い合わせ先にご連絡下さい（決済処理は行われていません）'
    export const EXCEED_MAX_ANNUAL_REWARDS_MESSAGE = '来年の１月１日までこのコンサルタントIDに対して相談を申し込むことは出来ません'
    export const NOT_ALL_CANDIDATES_ARE_INPUT_MESSAGE = '第一希望、第二希望、第三希望の相談開始日時すべてを指定した上でお申し込み下さい'
    export const REQUEST_CONSULTATION_SUCCESS_MESSAGE = '相談の申し込みを行いました。お申し込みされた内容を記載したメールを送りましたので、メールが届いているかご確認下さい'
    export const CARD_AUTH_PAYMENT_ERROR_MESSAGE = '相談申し込みに失敗しました。再度初めから相談申し込みを行って下さい'
    export const PAY_JP_CODE_INCORRECT_CARD_DATA_MESSAGE = 'カード情報に誤りがあります'
    export const PAY_JP_CODE_CARD_DECLINED_MESSAGE = 'ご利用のカードはカード会社に拒否されています。詳細はカード会社にご確認ください'
    export const PAY_JP_CODE_CARD_FLAGGED_MESSAGE = '該当カードの弊サービスにおける利用を一時的にロックしました。最後のエラーから24時間後に再度お試し下さい'
    export const PAY_JP_CODE_UNACCEPTABLE_BRAND_MESSAGE = 'ご利用のカードブランドはサポートされておりません'
    export const PAY_JP_CODE_THREE_D_SECURE_INCOMPLETED_MESSAGE = '相談申し込み中に別の操作が行われたため、処理に失敗しました。再度初めから相談申し込みを行って下さい'
    export const PAY_JP_CODE_THREE_D_SECURE_FAILED_MESSAGE = 'カードの認証処理（3Dセキュア認証）に失敗しました。理再度初めから相談申し込みを行って下さい'
    export const PAY_JP_CODE_NOT_IN_THREE_D_SECURE_FLOW_MESSAGE = 'カードの認証処理（3Dセキュア認証）がタイムアウトしました。再度初めから相談申し込みを行って下さい'
    export const CONSULTATION_REQUEST_REJECTION_MESSAGE = '相談申し込みを拒否しました'
    export const CONSULTATION_REQUEST_ACCEPTANCE_MESSAGE = '相談申し込みを受けました'
    export const NON_POSITIVE_CONSULTATION_REQ_ID_MESSAGE = '不正な相談申し込み番号です'
    export const NO_CONSULTATION_REQ_FOUND_MESSAGE = '相談申し込みが見つかりませんでした'
    export const INVALID_CANDIDATE_MESSAGE = '不正な相談日時候補です'
    export const USER_DOES_NOT_CHECK_CONFIRMATION_ITEMS_MESSAGE = '確認事項がチェックされていません'
    export const CONSULTANT_IS_NOT_AVAILABLE_ON_CONSULTATION_ACCEPTANCE_MESSAGE = 'アカウントが存在しない、または無効化されています' // 操作者が相談申し込みを受けるときに発生するメッセージなので、操作者自身のアカウントを示すエラーメッセージとなる
    export const USER_IS_NOT_AVAILABLE_ON_CONSULTATION_ACCEPTANCE_MESSAGE = '相談申し込み者のユーザーが存在しない、または無効化されています'
    export const USER_HAS_SAME_MEETING_DATE_TIME_MESSAGE = '選択された相談開始日時に関して、相談申し込み者が既に別のコンサルタントとの予定を確定させています。他の相談開始日時を選択して下さい'
    export const CONSULTANT_HAS_SAME_MEETING_DATE_TIME_MESSAGE = '選択された相談開始日時には既に相談予定が存在します。他の相談開始日時を選択して下さい' // 操作者が相談申し込みを受けるときに発生するメッセージなので、操作者自身の誤りを示すエラーメッセージとなる
    export const MEETING_DATE_TIME_OVERLAPS_MAINTENANCE_MESSAGE = '相談開始日時がメンテナンスと重なっています。他の相談開始日時を選択して下さい'
}
