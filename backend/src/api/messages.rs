pub mod error {
    pub const INTERNAL_SERVER: &str = "Lỗi hệ thống nội bộ";
    pub const INVALID_TOKEN: &str = "Mã định danh (Token) không hợp lệ";
    pub const AUTH_REQUIRED: &str = "Chưa được xác thực";
    pub const TOKEN_INVALID_OR_EXPIRED: &str = "Token không hợp lệ hoặc đã hết hạn";
    pub const ACCESS_DENIED: &str = "Bạn không có quyền thực hiện thao tác này";
    pub const INVALID_CREDENTIALS: &str = "Tài khoản hoặc mật khẩu không chính xác";
    pub const NOT_FRIEND_WITH_RECIPIENT: &str = "Bạn không phải bạn bè với người nhận";
    pub const NOT_CONVERSATION_MEMBER: &str = "Bạn không phải thành viên của cuộc trò chuyện này";
    pub const FILE_DELETE_FORBIDDEN: &str = "Bạn không có quyền xóa tệp này";

    pub const USER_NOT_FOUND: &str = "Không tìm thấy người dùng";
    pub const USER_INFO_NOT_FOUND: &str = "Không tìm thấy thông tin người dùng";
    pub const FRIEND_REQUEST_NOT_FOUND: &str = "Không tìm thấy yêu cầu kết bạn";
    pub const UPDATE_EMPTY_PAYLOAD: &str = "Không có trường dữ liệu nào cần cập nhật";
    pub const FRIEND_RECEIVER_NOT_FOUND: &str = "Không tìm thấy người dùng nhận";
    pub const FORBIDDEN_ACCEPT_FRIEND_REQUEST: &str =
        "Bạn không có quyền chấp nhận yêu cầu kết bạn này";
    pub const FORBIDDEN_DECLINE_FRIEND_REQUEST: &str =
        "Bạn không có quyền từ chối yêu cầu kết bạn này";

    pub const CONVERSATION_NOT_FOUND: &str = "Không tìm thấy cuộc trò chuyện";
    pub const MESSAGE_NOT_FOUND: &str = "Không tìm thấy tin nhắn";
    pub const FILE_NOT_FOUND: &str = "Không tìm thấy tệp";
    pub const CALL_NOT_FOUND: &str = "Không tìm thấy cuộc gọi";
    pub const REPLY_TARGET_NOT_FOUND: &str = "Tin nhắn được trả lời không tồn tại";
    pub const INVALID_PAGINATION_CURSOR: &str =
        "Định dạng danh sách phân trang (cursor) không hợp lệ";

    pub const CONVERSATION_MEMBER_REQUIRED: &str =
        "Cần có ít nhất một thành viên để tạo cuộc trò chuyện";
    pub const GROUP_CREATOR_MISSING: &str = "Lỗi dữ liệu nhóm: thiếu thông tin người tạo";
    pub const GROUP_DATA_ERROR: &str = "Lỗi dữ liệu nhóm";
    pub const GROUP_NOT_FOUND_OR_INVALID: &str = "Không tìm thấy nhóm hoặc không phải là nhóm";
    pub const ADDED_USER_NOT_FOUND: &str = "Không tìm thấy người dùng được thêm";

    pub const CLOUDINARY_NOT_CONFIGURED: &str = "Cloudinary chưa được cấu hình";
    pub const SYSTEM_TIMESTAMP_UNAVAILABLE: &str = "Không thể lấy timestamp hệ thống";
    pub const PASSWORD_HASH_FAILED: &str = "Lỗi tạo mật khẩu";
    pub const PASSWORD_VERIFY_FAILED: &str = "Lỗi xác thực mật khẩu";
    pub const CONFIG_SECRET_KEY_MISSING: &str = "Thiếu SECRET_KEY";
    pub const CONFIG_DATABASE_URL_MISSING: &str = "Thiếu DATABASE_URL";
    pub const CONFIG_REDIS_URL_MISSING: &str = "Thiếu REDIS_URL";
    pub const MIGRATION_FILES_LOAD_FAILED: &str = "Không thể tải migration files";
    pub const DATABASE_SCHEMA_INIT_FAILED: &str = "Lỗi khởi tạo Database schema";

    pub const MISSING_FILE_ATTACHMENT: &str = "Thiếu thông tin tệp đính kèm";
    pub const MISSING_FILE_NAME: &str = "Thiếu tên tệp";
    pub const INVALID_CURSOR: &str = "Cursor không hợp lệ";
    pub const MISSING_RECIPIENT_ID: &str = "Cần có ID người nhận";

    pub const ALREADY_FRIENDS: &str = "Hai người đã là bạn bè";
}

pub mod i18n {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Locale {
        Vi,
        En,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Key {
        InvalidToken,
        TokenInvalidOrExpired,
        AuthRequired,
        AccessDenied,
        UserNotFound,
        UserInfoNotFound,
        UpdateEmptyPayload,
        FriendReceiverNotFound,
        FriendRequestNotFound,
        ForbiddenAcceptFriendRequest,
        ForbiddenDeclineFriendRequest,
        InvalidCursor,
        MissingRecipientId,
        NotFriendWithRecipient,
        NotConversationMember,
        MissingFileAttachment,
        MissingFileName,
        FileNotFound,
        MessageNotFound,
        FileDeleteForbidden,
        CallNotFound,
        InvalidPaginationCursor,
        ConversationNotFound,
        ConversationMemberRequired,
        GroupCreatorMissing,
        GroupDataError,
        GroupNotFoundOrInvalid,
        AddedUserNotFound,
        CloudinaryNotConfigured,
        SystemTimestampUnavailable,
        PasswordHashFailed,
        PasswordVerifyFailed,
        ConfigSecretKeyMissing,
        ConfigDatabaseUrlMissing,
        ConfigRedisUrlMissing,
        MigrationFilesLoadFailed,
        DatabaseSchemaInitFailed,
        AlreadyFriends,
        InvalidCredentials,
        ReplyTargetNotFound,
        SearchQueryRequired,
        SearchQueryTooShort,
        DirectRecipientResolveFailed,
        FriendRequestAlreadyExists,
        SelfFriendRequestNotAllowed,
        GroupUpdateOnlyForGroup,
        GroupOwnerOnlyUpdate,
        GroupAddOnlyFriends,
        GroupOwnerOnlyAdd,
        GroupRemoveOnlyGroup,
        CannotRemoveGroupOwner,
        DeleteOwnMessageOnly,
        EditOwnMessageOnly,
        MessageDeletedOrNotFound,
        ReplyTargetWrongConversation,
        MessageContentOrFileRequired,
        MessageTypeRequiresFileUrl,
        TextMessageRequiresContent,
        CallResponseNotAllowed,
        CallNotAwaitingResponse,
        CallCancelInitiatorOnly,
        CallCancelInvalidStatus,
        CallEndNotAllowed,
        UserUpdateSelfOnly,
        UserDeleteSelfOnly,
        PresenceMaxUsersExceeded,
        NotFriendsWithAllMembers,
        CloudinaryDeleteFailed,
        ConfigSecretKeyEmpty,
        ConfigTokenExpirationInvalid,
        ConfigDatabaseRedisEmpty,
        InternalServer,
    }

    pub fn detect_locale(accept_language: Option<&str>) -> Locale {
        let Some(raw) = accept_language else {
            return Locale::Vi;
        };

        let normalized = raw.to_ascii_lowercase();
        if normalized.starts_with("en") {
            Locale::En
        } else {
            Locale::Vi
        }
    }

    pub fn t(locale: Locale, key: Key) -> &'static str {
        match (locale, key) {
            (Locale::Vi, Key::InvalidToken) => "Mã định danh (Token) không hợp lệ",
            (Locale::En, Key::InvalidToken) => "Invalid token",

            (Locale::Vi, Key::TokenInvalidOrExpired) => "Token không hợp lệ hoặc đã hết hạn",
            (Locale::En, Key::TokenInvalidOrExpired) => "Token is invalid or expired",

            (Locale::Vi, Key::AuthRequired) => "Chưa được xác thực",
            (Locale::En, Key::AuthRequired) => "Authentication required",

            (Locale::Vi, Key::AccessDenied) => "Bạn không có quyền thực hiện thao tác này",
            (Locale::En, Key::AccessDenied) => "You are not allowed to perform this action",

            (Locale::Vi, Key::UserNotFound) => "Không tìm thấy người dùng",
            (Locale::En, Key::UserNotFound) => "User not found",

            (Locale::Vi, Key::UserInfoNotFound) => "Không tìm thấy thông tin người dùng",
            (Locale::En, Key::UserInfoNotFound) => "User information not found",

            (Locale::Vi, Key::UpdateEmptyPayload) => "Không có trường dữ liệu nào cần cập nhật",
            (Locale::En, Key::UpdateEmptyPayload) => "No fields provided for update",

            (Locale::Vi, Key::FriendReceiverNotFound) => "Không tìm thấy người dùng nhận",
            (Locale::En, Key::FriendReceiverNotFound) => "Friend request receiver not found",

            (Locale::Vi, Key::FriendRequestNotFound) => "Không tìm thấy yêu cầu kết bạn",
            (Locale::En, Key::FriendRequestNotFound) => "Friend request not found",

            (Locale::Vi, Key::ForbiddenAcceptFriendRequest) => {
                "Bạn không có quyền chấp nhận yêu cầu kết bạn này"
            }
            (Locale::En, Key::ForbiddenAcceptFriendRequest) => {
                "You are not allowed to accept this friend request"
            }

            (Locale::Vi, Key::ForbiddenDeclineFriendRequest) => {
                "Bạn không có quyền từ chối yêu cầu kết bạn này"
            }
            (Locale::En, Key::ForbiddenDeclineFriendRequest) => {
                "You are not allowed to decline this friend request"
            }

            (Locale::Vi, Key::InvalidCursor) => "Cursor không hợp lệ",
            (Locale::En, Key::InvalidCursor) => "Invalid cursor",

            (Locale::Vi, Key::InvalidPaginationCursor) => {
                "Định dạng danh sách phân trang (cursor) không hợp lệ"
            }
            (Locale::En, Key::InvalidPaginationCursor) => "Invalid pagination cursor format",

            (Locale::Vi, Key::ConversationNotFound) => "Không tìm thấy cuộc trò chuyện",
            (Locale::En, Key::ConversationNotFound) => "Conversation not found",

            (Locale::Vi, Key::ConversationMemberRequired) => {
                "Cần có ít nhất một thành viên để tạo cuộc trò chuyện"
            }
            (Locale::En, Key::ConversationMemberRequired) => {
                "At least one member is required to create a conversation"
            }

            (Locale::Vi, Key::GroupCreatorMissing) => "Lỗi dữ liệu nhóm: thiếu thông tin người tạo",
            (Locale::En, Key::GroupCreatorMissing) => {
                "Group data error: missing creator information"
            }

            (Locale::Vi, Key::GroupDataError) => "Lỗi dữ liệu nhóm",
            (Locale::En, Key::GroupDataError) => "Group data error",

            (Locale::Vi, Key::GroupNotFoundOrInvalid) => {
                "Không tìm thấy nhóm hoặc không phải là nhóm"
            }
            (Locale::En, Key::GroupNotFoundOrInvalid) => {
                "Group not found or invalid group conversation"
            }

            (Locale::Vi, Key::AddedUserNotFound) => "Không tìm thấy người dùng được thêm",
            (Locale::En, Key::AddedUserNotFound) => "Added user not found",

            (Locale::Vi, Key::CloudinaryNotConfigured) => "Cloudinary chưa được cấu hình",
            (Locale::En, Key::CloudinaryNotConfigured) => "Cloudinary is not configured",

            (Locale::Vi, Key::SystemTimestampUnavailable) => "Không thể lấy timestamp hệ thống",
            (Locale::En, Key::SystemTimestampUnavailable) => "Unable to read system timestamp",

            (Locale::Vi, Key::PasswordHashFailed) => "Lỗi tạo mật khẩu",
            (Locale::En, Key::PasswordHashFailed) => "Failed to hash password",

            (Locale::Vi, Key::PasswordVerifyFailed) => "Lỗi xác thực mật khẩu",
            (Locale::En, Key::PasswordVerifyFailed) => "Failed to verify password",

            (Locale::Vi, Key::ConfigSecretKeyMissing) => "Thiếu SECRET_KEY",
            (Locale::En, Key::ConfigSecretKeyMissing) => "Missing SECRET_KEY",

            (Locale::Vi, Key::ConfigDatabaseUrlMissing) => "Thiếu DATABASE_URL",
            (Locale::En, Key::ConfigDatabaseUrlMissing) => "Missing DATABASE_URL",

            (Locale::Vi, Key::ConfigRedisUrlMissing) => "Thiếu REDIS_URL",
            (Locale::En, Key::ConfigRedisUrlMissing) => "Missing REDIS_URL",

            (Locale::Vi, Key::MigrationFilesLoadFailed) => "Không thể tải migration files",
            (Locale::En, Key::MigrationFilesLoadFailed) => "Unable to load migration files",

            (Locale::Vi, Key::DatabaseSchemaInitFailed) => "Lỗi khởi tạo Database schema",
            (Locale::En, Key::DatabaseSchemaInitFailed) => "Failed to initialize database schema",

            (Locale::Vi, Key::AlreadyFriends) => "Hai người đã là bạn bè",
            (Locale::En, Key::AlreadyFriends) => "Users are already friends",

            (Locale::Vi, Key::InvalidCredentials) => "Tài khoản hoặc mật khẩu không chính xác",
            (Locale::En, Key::InvalidCredentials) => "Invalid username or password",

            (Locale::Vi, Key::ReplyTargetNotFound) => "Tin nhắn được trả lời không tồn tại",
            (Locale::En, Key::ReplyTargetNotFound) => "Replied message does not exist",

            (Locale::Vi, Key::MissingRecipientId) => "Cần có ID người nhận",
            (Locale::En, Key::MissingRecipientId) => "Recipient ID is required",

            (Locale::Vi, Key::NotFriendWithRecipient) => "Bạn không phải bạn bè với người nhận",
            (Locale::En, Key::NotFriendWithRecipient) => "You are not friends with the recipient",

            (Locale::Vi, Key::NotConversationMember) => {
                "Bạn không phải thành viên của cuộc trò chuyện này"
            }
            (Locale::En, Key::NotConversationMember) => "You are not a member of this conversation",

            (Locale::Vi, Key::MissingFileAttachment) => "Thiếu thông tin tệp đính kèm",
            (Locale::En, Key::MissingFileAttachment) => "Missing file attachment information",

            (Locale::Vi, Key::MissingFileName) => "Thiếu tên tệp",
            (Locale::En, Key::MissingFileName) => "Missing file name",

            (Locale::Vi, Key::FileNotFound) => "Không tìm thấy tệp",
            (Locale::En, Key::FileNotFound) => "File not found",

            (Locale::Vi, Key::MessageNotFound) => "Không tìm thấy tin nhắn",
            (Locale::En, Key::MessageNotFound) => "Message not found",

            (Locale::Vi, Key::FileDeleteForbidden) => "Bạn không có quyền xóa tệp này",
            (Locale::En, Key::FileDeleteForbidden) => "You are not allowed to delete this file",

            (Locale::Vi, Key::CallNotFound) => "Không tìm thấy cuộc gọi",
            (Locale::En, Key::CallNotFound) => "Call not found",

            (Locale::Vi, Key::SearchQueryRequired) => "Từ khóa tìm kiếm không được để trống",
            (Locale::En, Key::SearchQueryRequired) => "Search query must not be empty",

            (Locale::Vi, Key::SearchQueryTooShort) => "Từ khóa tìm kiếm phải có ít nhất 2 ký tự",
            (Locale::En, Key::SearchQueryTooShort) => "Search query must be at least 2 characters",

            (Locale::Vi, Key::DirectRecipientResolveFailed) => {
                "Không thể xác định người nhận trong cuộc trò chuyện trực tiếp"
            }
            (Locale::En, Key::DirectRecipientResolveFailed) => {
                "Unable to resolve recipient in direct conversation"
            }

            (Locale::Vi, Key::FriendRequestAlreadyExists) => "Yêu cầu kết bạn đã tồn tại",
            (Locale::En, Key::FriendRequestAlreadyExists) => "Friend request already exists",

            (Locale::Vi, Key::SelfFriendRequestNotAllowed) => {
                "Không thể tự gửi yêu cầu kết bạn cho chính mình"
            }
            (Locale::En, Key::SelfFriendRequestNotAllowed) => {
                "Cannot send a friend request to yourself"
            }

            (Locale::Vi, Key::GroupUpdateOnlyForGroup) => "Chỉ có thể cập nhật thông tin cho nhóm",
            (Locale::En, Key::GroupUpdateOnlyForGroup) => "Only group conversations can be updated",

            (Locale::Vi, Key::GroupOwnerOnlyUpdate) => {
                "Chỉ trưởng nhóm mới có quyền thay đổi thông tin"
            }
            (Locale::En, Key::GroupOwnerOnlyUpdate) => {
                "Only the group owner can update group information"
            }

            (Locale::Vi, Key::GroupAddOnlyFriends) => "Chỉ có thể thêm bạn bè vào nhóm",
            (Locale::En, Key::GroupAddOnlyFriends) => "Only friends can be added to the group",

            (Locale::Vi, Key::GroupOwnerOnlyAdd) => "Chỉ trưởng nhóm mới có quyền thêm thành viên",
            (Locale::En, Key::GroupOwnerOnlyAdd) => "Only the group owner can add members",

            (Locale::Vi, Key::GroupRemoveOnlyGroup) => {
                "Chỉ có thể rời nhóm hoặc xóa thành viên khỏi nhóm"
            }
            (Locale::En, Key::GroupRemoveOnlyGroup) => {
                "Only group conversations support removing members or leaving"
            }

            (Locale::Vi, Key::CannotRemoveGroupOwner) => "Không thể xóa trưởng nhóm.",
            (Locale::En, Key::CannotRemoveGroupOwner) => "Cannot remove group owner.",

            (Locale::Vi, Key::DeleteOwnMessageOnly) => "Bạn chỉ có thể xóa tin nhắn của chính mình",
            (Locale::En, Key::DeleteOwnMessageOnly) => "You can only delete your own messages",

            (Locale::Vi, Key::EditOwnMessageOnly) => {
                "Bạn chỉ có thể chỉnh sửa tin nhắn của chính mình"
            }
            (Locale::En, Key::EditOwnMessageOnly) => "You can only edit your own messages",

            (Locale::Vi, Key::MessageDeletedOrNotFound) => {
                "Không tìm thấy tin nhắn hoặc tin nhắn đã bị xóa"
            }
            (Locale::En, Key::MessageDeletedOrNotFound) => "Message not found or already deleted",

            (Locale::Vi, Key::ReplyTargetWrongConversation) => {
                "Tin nhắn được trả lời không thuộc cuộc trò chuyện này"
            }
            (Locale::En, Key::ReplyTargetWrongConversation) => {
                "Reply target does not belong to this conversation"
            }

            (Locale::Vi, Key::MessageContentOrFileRequired) => {
                "Tin nhắn phải có nội dung hoặc tệp đính kèm"
            }
            (Locale::En, Key::MessageContentOrFileRequired) => {
                "Message requires content or file attachment"
            }

            (Locale::Vi, Key::MessageTypeRequiresFileUrl) => "Loại tin nhắn này yêu cầu file_url",
            (Locale::En, Key::MessageTypeRequiresFileUrl) => "This message type requires file_url",

            (Locale::Vi, Key::TextMessageRequiresContent) => "Tin nhắn văn bản yêu cầu nội dung",
            (Locale::En, Key::TextMessageRequiresContent) => "Text message requires content",

            (Locale::Vi, Key::CallResponseNotAllowed) => "Bạn không thể phản hồi cuộc gọi này",
            (Locale::En, Key::CallResponseNotAllowed) => "You cannot respond to this call",

            (Locale::Vi, Key::CallNotAwaitingResponse) => {
                "Cuộc gọi không còn ở trạng thái chờ phản hồi"
            }
            (Locale::En, Key::CallNotAwaitingResponse) => {
                "Call is no longer waiting for a response"
            }

            (Locale::Vi, Key::CallCancelInitiatorOnly) => "Chỉ người gọi mới có thể hủy cuộc gọi",
            (Locale::En, Key::CallCancelInitiatorOnly) => "Only the caller can cancel the call",

            (Locale::Vi, Key::CallCancelInvalidStatus) => {
                "Không thể hủy cuộc gọi ở trạng thái hiện tại"
            }
            (Locale::En, Key::CallCancelInvalidStatus) => "Cannot cancel call in current status",

            (Locale::Vi, Key::CallEndNotAllowed) => "Bạn không thể kết thúc cuộc gọi này",
            (Locale::En, Key::CallEndNotAllowed) => "You cannot end this call",

            (Locale::Vi, Key::UserUpdateSelfOnly) => {
                "Bạn chỉ có thể cập nhật thông tin của chính mình"
            }
            (Locale::En, Key::UserUpdateSelfOnly) => "You can only update your own profile",

            (Locale::Vi, Key::UserDeleteSelfOnly) => "Bạn chỉ có thể xóa tài khoản của chính mình",
            (Locale::En, Key::UserDeleteSelfOnly) => "You can only delete your own account",

            (Locale::Vi, Key::PresenceMaxUsersExceeded) => {
                "Tối đa 200 user IDs cho mỗi lần gọi API"
            }
            (Locale::En, Key::PresenceMaxUsersExceeded) => "Maximum 200 user IDs per request",

            (Locale::Vi, Key::NotFriendsWithAllMembers) => {
                "Bạn không phải bạn bè với tất cả các thành viên"
            }
            (Locale::En, Key::NotFriendsWithAllMembers) => "You are not friends with all members",

            (Locale::Vi, Key::CloudinaryDeleteFailed) => "Không thể xóa file trên Cloudinary",
            (Locale::En, Key::CloudinaryDeleteFailed) => "Unable to delete file on Cloudinary",

            (Locale::Vi, Key::ConfigSecretKeyEmpty) => "SECRET_KEY không được để trống",
            (Locale::En, Key::ConfigSecretKeyEmpty) => "SECRET_KEY must not be empty",

            (Locale::Vi, Key::ConfigTokenExpirationInvalid) => "Token expiration phải lớn hơn 0",
            (Locale::En, Key::ConfigTokenExpirationInvalid) => {
                "Token expiration must be greater than 0"
            }

            (Locale::Vi, Key::ConfigDatabaseRedisEmpty) => {
                "DATABASE_URL và REDIS_URL không được để trống"
            }
            (Locale::En, Key::ConfigDatabaseRedisEmpty) => {
                "DATABASE_URL and REDIS_URL must not be empty"
            }

            (Locale::Vi, Key::InternalServer) => "Có lỗi nội bộ xảy ra",
            (Locale::En, Key::InternalServer) => "Internal server error",
        }
    }
}
