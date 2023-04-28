// Copyright 2021 Ken Miura

pub(crate) mod document_operation;
pub(crate) mod login_status;
pub(crate) mod terms_of_use;

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {

    use axum::async_trait;
    use common::{smtp::SendMail, ErrResp};

    #[derive(Clone, Debug)]
    pub(crate) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(crate) fn new(to: String, from: String, subject: String, text: String) -> Self {
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
