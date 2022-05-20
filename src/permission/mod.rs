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

const GROUP_LIMIT: u32 = 16;
const PERM_LIMIT: u32 = u64::BITS;
