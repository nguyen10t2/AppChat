pub fn validation_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn validation_password(password: &str) -> bool {
    password.len() >= 6
}

pub fn validation_fullname(fullname: &str) -> bool {
    !fullname.trim().is_empty()
}

pub fn validation_otp(otp: &str) -> bool {
    otp.len() == 6 && otp.chars().all(|c| c.is_digit(10))
}