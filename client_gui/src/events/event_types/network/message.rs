use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MessageEvent {
    pub author: String,
    pub text: String,
}

impl MessageEvent {
    pub fn new(data: (String, String)) -> MessageEvent {
        MessageEvent { author: data.0, text: data.1 }
    }
}
