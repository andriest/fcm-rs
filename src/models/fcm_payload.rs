#[derive(Debug)]
pub struct FCMPayloadData {
    pub title: String,
    pub message: String,
    pub image: String,
    pub click_action: String,
}
