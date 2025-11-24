use actix_web::{web, error::ErrorUnauthorized};
use actix_web::error::Error;
use mongodb::bson::oid::ObjectId;
use serde_json::json;
use crate::services::friend_service::FriendService;

fn swap_objectid(a: &ObjectId, b: &ObjectId) -> (ObjectId, ObjectId) {
    if a < b {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    }
}

pub async fn verify_friendship(
    // Service để truy cập DB
    friend_service: &web::Data<FriendService>,
    sender_id: &ObjectId,
    recipient_id: &ObjectId,
) -> Result<(), Error> {
    
    let (user1, user2) = swap_objectid(sender_id, recipient_id);
    
    match friend_service.find_one(&user1, &user2).await {
        Ok(Some(_friendship_status)) => {
            // Kiểm tra thành công: Là bạn bè
            Ok(())
        }
        Ok(None) => {
            Err(ErrorUnauthorized(json!({
                "error": "Bạn không có quyền gửi tin nhắn cho người này",
            })))
        }
        Err(e) => {
            Err(ErrorUnauthorized(json!({
                "error": format!("Lỗi khi kiểm tra trạng thái bạn bè: {}", e),
            })))
        }
    }
}