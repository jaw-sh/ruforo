use once_cell::sync::OnceCell;
use ruforo::permission::{Category, CategoryValues, Flag};

const HB_CHAT_PERMS: &[&str] = &[
    "hbChatMessageDeleteOther",
    "hbChatMessageDeleteOwn",
    "hbChatMessageEditOther",
    "hbChatMessageEditOwn",
    "hbChatMessageReport",
    "hbChatMessageSend",
    "hbChatMessageUndelete",
    "hbChatMessageViewDeleted",
    "hbChatRoomMotd",
    "hbChatRoomView",
];

static XF_PERMISSIONS: OnceCell<Category> = OnceCell::new();

#[inline(always)]
pub fn get_permissions() -> &'static Category {
    unsafe { XF_PERMISSIONS.get_unchecked() }
}

pub fn configure() {
    let mut category = Category::default();

    for (i, perm) in HB_CHAT_PERMS.into_iter().enumerate() {
        if let Err(_) = category.add_item((i + 1) as i32, perm) {
            log::warn!("XF Perm Config Err");
        }
    }

    if XF_PERMISSIONS.set(category).is_err() {
        panic!("failed to set XF_PERMISSION");
    }
}

pub fn json_to_values(json: serde_json::Value) -> CategoryValues {
    let mut category_values = CategoryValues::default();
    let perms = get_permissions();

    for perm in HB_CHAT_PERMS {
        if let Some(value) = json.get(perm) {
            if let Some(value) = value.as_bool() {
                category_values.set_flag(
                    perms
                        .borrow_item_by_label(perm)
                        .expect("Unable to unwrap permission by label")
                        .position,
                    match value {
                        true => Flag::YES,
                        false => Flag::NO,
                    },
                )
            }
        }
    }

    category_values
}
