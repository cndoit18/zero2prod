use std::time::Duration;

use crate::domain::SubscriberEmail;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

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
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        let email = Message::builder()
            .from(self.sender.as_ref().parse().unwrap())
            .to(recipient.as_ref().parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative() // This is composed of two parts.
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(text_content.to_string()), // Every message should have a plain text fallback.
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(html_content.to_string()),
                    ),
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
    use fake::faker::lorem::raw::{Paragraph, Sentence};
    use fake::locales::EN;
    use fake::Fake;
    use maik::MockServer;
    use url::Url;

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let sender_email = SubscriberEmail::parse(SafeEmail(EN).fake::<String>()).unwrap();
        let sender_password = Password(EN, 8..15).fake::<String>();

        let mut mock_server = MockServer::new(DomainSuffix(EN).fake::<String>().as_str());
        mock_server.add_mailbox(sender_email.as_ref(), sender_password.as_ref());
        mock_server.start();
        let email_client = EmailClient::new(
            Url::parse(&format!(
                "smtp://{}:{}@{}:{}",
                sender_email.as_ref(),
                sender_password.as_str(),
                mock_server.host(),
                mock_server.port()
            ))
            .unwrap()
            .as_str(),
            sender_email,
            Duration::from_secs(10),
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();
        let subject = Sentence(EN, 1..2).fake::<String>();
        let content = Paragraph(EN, 1..10).fake::<String>();
        email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await
            .unwrap();
    }
}
