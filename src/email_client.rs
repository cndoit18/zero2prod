use std::time::Duration;

use crate::domain::SubscriberEmail;
use lettre::message::{header, Mailbox, SinglePart};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[allow(dead_code)]
pub struct EmailClient {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(smtp_url: &str, sender: SubscriberEmail, timeout: Duration) -> Self {
        Self {
            mailer: AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_url)
                .unwrap()
                .timeout(Some(timeout))
                .build(),
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        text_content: &str,
    ) -> Result<(), String> {
        let address: Address = self.sender.as_ref().parse().unwrap();
        let email = Message::builder()
            .from(Mailbox {
                name: Some(address.user().to_string()),
                email: address,
            })
            .to(recipient.as_ref().parse().unwrap())
            .subject(subject)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(text_content.to_string()), // Every message should have a plain text fallback.
            )
            .map_err(|err| format!("failed to build email: {}", err))?;

        self.mailer
            .send(email)
            .await
            .map_err(|err| format!("failed to send email: {}", err))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fake::faker::internet::raw::{DomainSuffix, Password, SafeEmail};
    use fake::faker::lorem::raw::Sentence;
    use fake::locales::EN;
    use fake::Fake;
    use maik::{MailAssertion, MockServer};
    use url::Url;

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mut mock_server = MockServer::new(DomainSuffix(EN).fake::<String>().as_str());
        // let email = SubscriberEmail::parse(SafeEmail(EN).fake::<String>()).unwrap();
        let email = SubscriberEmail::parse(SafeEmail(EN).fake::<String>()).unwrap();
        let password = Password(EN, 8..15).fake::<String>();
        mock_server.add_mailbox(email.as_ref(), password.as_ref());
        mock_server.start();
        let email_client = EmailClient::new(
            Url::parse(&format!(
                "smtp://{}:{}@{}:{}",
                email.as_ref(),
                password.as_str(),
                mock_server.host(),
                mock_server.port()
            ))
            .unwrap()
            .as_str(),
            email.clone(),
            Duration::from_secs(10),
        );
        let subscriber_email = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();
        let subject = Sentence(EN, 4..5).fake::<String>();
        let content = Sentence(EN, 8..10).fake::<String>();
        email_client
            .send_email(subscriber_email, &subject, &content)
            .await
            .unwrap();
        assert!(mock_server.assert(MailAssertion::new().sender_is(email.as_ref())),);
    }
}
