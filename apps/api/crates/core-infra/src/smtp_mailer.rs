//! Email delivery. `SmtpMailer` sends plaintext mail over SMTP (Mailpit in dev,
//! a real relay in prod). `NoopMailer` just logs — used when SMTP isn't configured.

use core_domain::ports::{EmailError, EmailSender, OutgoingEmail};
use lettre::message::Mailbox;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::{Address, AsyncTransport, Message, Tokio1Executor};

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpMailer {
    /// Build a mailer pointing at `host:port`. Uses an unencrypted connection
    /// (`builder_dangerous`) — correct for local Mailpit; front a TLS relay in prod.
    pub fn new(
        host: &str,
        port: u16,
        from_address: &str,
        from_name: &str,
    ) -> Result<Self, EmailError> {
        let address: Address = from_address
            .parse()
            .map_err(|e| EmailError(format!("invalid EMAIL_FROM '{from_address}': {e}")))?;
        let from = Mailbox::new(Some(from_name.to_string()), address);
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .build();
        Ok(Self { transport, from })
    }
}

#[async_trait::async_trait]
impl EmailSender for SmtpMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<(), EmailError> {
        let to: Address = email
            .to_address
            .parse()
            .map_err(|e| EmailError(format!("invalid recipient '{}': {e}", email.to_address)))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(Mailbox::new(Some(email.to_name.clone()), to))
            .subject(email.subject.clone())
            .body(email.body.clone())
            .map_err(|e| EmailError(e.to_string()))?;
        self.transport
            .send(message)
            .await
            .map_err(|e| EmailError(e.to_string()))?;
        Ok(())
    }
}

/// No-op mailer: logs what would be sent. Used when `SMTP_HOST` is unset.
#[derive(Default, Clone)]
pub struct NoopMailer;

#[async_trait::async_trait]
impl EmailSender for NoopMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<(), EmailError> {
        tracing::info!(to = %email.to_address, subject = %email.subject, "email disabled — would send");
        Ok(())
    }
}
