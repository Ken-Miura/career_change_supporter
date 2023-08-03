// Copyright 2023 Ken Miura

pub(crate) mod account_creation;
pub(crate) mod health;
pub(crate) mod news;
pub(crate) mod session;

pub(super) const ROOT_PATH: &str = "/api";

#[cfg(test)]
pub(super) mod tests {

    use axum::async_trait;
    use common::{smtp::SendMail, ErrResp};

    #[derive(Clone, Debug)]
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
