#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TCP_lift_order_t {
    pub button_type: ButtonType,
    pub floor: u32,
    pub elevator_id: u32,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum tcp_message {
    set_order { order: TCP_lift_order_t },   // a single order
    clear_order { order: TCP_lift_order_t }, // a single order is to be cleared.
    NOP { elevator_id: u32 },                //do nothing
}