use crate::game::{pfnUserMsgHook, user_msg_s};

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::USER_MSG;
// END MUTABLE GLOBAL STATE

pub struct Hook {
}

impl Hook {
    pub fn new() -> Self {
        Self {}
    }
}

impl Drop for Hook {
    fn drop(&mut self) {

    }
}

impl user_msg_s {
    fn find(name: &str) -> Option<Self> {
        None
    }
}

fn hook_user_msg(message_name: &str, hook: pfnUserMsgHook) {

}