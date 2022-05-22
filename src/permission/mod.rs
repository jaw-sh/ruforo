pub mod category;
pub mod category_values;
pub mod collection;
pub mod collection_values;
pub mod error;
pub mod flag;
pub mod item;
pub mod item_values;
pub mod mask;
pub mod resource;
mod test;

/// Maximum number of permission categories
const GROUP_LIMIT: u32 = 16;
/// Maximum number of permissions per category (64 bits)
const PERM_LIMIT: u32 = u64::BITS;

use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct PermissionData {
    collection: Arc<collection::Collection>,
    collection_values: DashMap<(i32, i32), collection_values::CollectionValues>,
}

pub fn init() {}
