// Copyright 2023 Ken Miura

pub(crate) mod admin;
pub(crate) mod career_request;
pub(crate) mod identity_by_user_account_id;
pub(crate) mod identity_request;
pub(crate) mod pagination;
mod reason_validator;
pub(crate) mod refresh;
pub(crate) mod user_account_info;
mod user_account_operation;

#[cfg(test)]
pub(super) mod tests {

    use axum::async_trait;

    use common::{smtp::SendMail, ErrResp};

    pub(super) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(super) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    #[async_trait]
    impl SendMail for SendMailMock {
        async fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }
}
