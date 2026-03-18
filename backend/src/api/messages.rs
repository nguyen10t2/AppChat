pub mod error {
    pub const INTERNAL_SERVER: &str = "Lỗi hệ thống nội bộ";
    pub const INVALID_TOKEN: &str = "Mã định danh (Token) không hợp lệ";
    pub const USER_NOT_FOUND: &str = "Không tìm thấy người dùng";
    pub const FRIEND_REQUEST_NOT_FOUND: &str = "Không tìm thấy yêu cầu kết bạn";
    pub const UPDATE_EMPTY_PAYLOAD: &str = "Không có trường dữ liệu nào cần cập nhật";
    pub const FRIEND_RECEIVER_NOT_FOUND: &str = "Không tìm thấy người dùng nhận";
    pub const FORBIDDEN_ACCEPT_FRIEND_REQUEST: &str =
        "Bạn không có quyền chấp nhận yêu cầu kết bạn này";
    pub const FORBIDDEN_DECLINE_FRIEND_REQUEST: &str =
        "Bạn không có quyền từ chối yêu cầu kết bạn này";
}
