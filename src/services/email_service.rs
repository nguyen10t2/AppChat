use lettre::transport::smtp::SmtpTransport;
use lettre::{Message, Transport};
use std::env;

pub struct EmailService {
    smtp_email: String,
    smtp_password: String,
    app_url: String,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            smtp_email: env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
            app_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
        }
    }

    pub async fn send_otp_email(
        &self,
        to: &str,
        otp: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let html_body = format!(
            r#"
        <h2>Verify Your Email</h2>
        <p>Your OTP code is: <strong>{}</strong></p>
        <p>Or click <a href="{}/register/otp">here</a> to verify</p>
        <p>OTP code is valid for 10 minutes</p>
        "#,
            otp, self.app_url
        );

        let email = Message::builder()
            .from(self.smtp_email.parse()?)
            .to(to.parse()?)
            .subject("Your OTP Code")
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(lettre::message::SinglePart::plain(format!(
                        "Your OTP: {}",
                        otp
                    )))
                    .singlepart(lettre::message::SinglePart::html(html_body)),
            )?;

        let smtp = SmtpTransport::relay("smtp.gmail.com")?
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                self.smtp_email.clone(),
                self.smtp_password.clone(),
            ))
            .timeout(Some(std::time::Duration::from_secs(10)))
            .build();

        actix_web::web::block(move || smtp.send(&email))
            .await
            .map_err(|e| format!("Blocking error: {}", e))??;

        Ok(())
    }
}
