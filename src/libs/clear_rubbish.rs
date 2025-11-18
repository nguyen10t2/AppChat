use actix_web::web;
use tokio::time::{interval, Duration};

use crate::services::otp_service::OtpService;

pub async fn start_cleanup_task(
    otp_service: web::Data<OtpService>,
) {
    let mut interval = interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;
        println!("Đang dọng rác...");

        if let Err(e) = otp_service.delete_otp().await {
            eprintln!("Lỗi khi dọn rác: {:?}", e);
        }

        println!("Dọn dẹp hoàn tất");
    }
}