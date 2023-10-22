use crate::types::payload::Payload;

pub enum MessageResponse {
    NoAck {
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        payload: Payload,
    },
    Ack {
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        payload: Payload,
    },
    Response {
        payload: Payload,
    },
    ResponseWithAck {
        payload: Payload,
    },
}
