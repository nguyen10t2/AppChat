use actix_web::web;
use tokio::time::{interval, Duration};

use crate::services::{session_service::SessionService, otp_service::OtpService};

pub async fn start_cleanup_task(
    session_service: web::Data<SessionService>,
    otp_service: web::Data<OtpService>,
) {
    let mut interval = interval(Duration::from_secs(3600)); // 3600s = 1 giờ

    loop {
        interval.tick().await;
        println!("Running cleanup...");

        if let Err(e) = session_service.cleanup_expired().await {
            eprintln!("Error cleaning sessions: {:?}", e);
        }

        if let Err(e) = otp_service.delete_otp().await {
            eprintln!("Error cleaning OTPs: {:?}", e);
        }

        println!("Cleanup finished");
    }
}