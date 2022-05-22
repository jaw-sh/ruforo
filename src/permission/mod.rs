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
/// Total maximum number of permissions defined as GROUP_LIMIT*PERM_LIMIT
const MAX_PERMS: u32 = GROUP_LIMIT * PERM_LIMIT;

use actix::fut::stream::Collect;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct PermissionData {
    /// Threadsafe Data Structure
    collection: Arc<collection::Collection>,
    /// (Group, User) -> CollectionValues Relationship
    collection_values: DashMap<(i32, i32), collection_values::CollectionValues>,
}

pub async fn init() -> Result<PermissionData, sea_orm::error::DbErr> {
    use crate::init::get_db_pool;
    use crate::orm::permission_collections;
    use crate::orm::permission_values;
    use crate::orm::permissions;
    use collection_values::CollectionValues;
    use sea_orm::{entity::*, query::*};

    // Build structure tree
    let mut col = collection::Collection::default();

    // Import permissions
    let items = permissions::Entity::find().all(get_db_pool()).await?;

    // Pull unique category id list from permissions.
    let mut ucid: Vec<i32> = items.iter().map(|i| i.category_id).collect();
    ucid.sort_unstable();
    ucid.dedup();

    // Add categories to collection and order them.
    for (i, cid) in ucid.iter().enumerate() {
        col.categories[i].id = *cid;
        col.categories[i].position = i as u8;

        // Add permissions belonging to this category.
        for item in items.iter() {
            if *cid == item.category_id {
                match col.categories[i].add_item(&item.category_id, &item.label) {
                    Ok(item) => {
                        col.dictionary.insert(
                            item.label.to_owned(),
                            (item.category as u8, item.position as u8),
                        );
                        col.lookup
                            .insert(item.id, (item.category as u8, item.position as u8));
                    }
                    Err(_) => {
                        println!("Category overflow adding {:?}", item);
                    }
                }
            }
        }
    }

    // Import data
    let mut vals: DashMap<(i32, i32), CollectionValues> = Default::default();
    let perms: Vec<(permission_collections::Model, Vec<permission_values::Model>)> =
        permission_collections::Entity::find()
            .find_with_related(permission_values::Entity)
            .all(get_db_pool())
            .await?;

    // convert ORM data into permission system structs
    // loop through the collection-<values relations
    for pt in perms.iter() {
        // Split ORMs into Collection -> Values
        let (pc, pvs) = pt;
        // Create collection values record to set flags on
        let mut cv = CollectionValues::default();

        // loop through the values
        for pv in pvs.iter() {
            // Look up the permissions's indices by id.
            if let Some(pindices) = col.lookup.get(&pv.permission_id) {
                // Assign each flag to the CollectionValues.
                cv.set_flag(&pindices.0, &pindices.1, &pv.value);
            } else {
                println!(
                    "Failed to lookup indices for permission_values {:?},{:?}",
                    pv.collection_id, pv.permission_id
                );
            }
        }

        //let val_key: (i32, i32) = (pc.group_id.unwrap_or(0), pc.user_id.unwrap_or(0));
        //let cvs = vals.insert(val_key, cv);
    }

    Ok(PermissionData {
        collection: Arc::new(col),
        collection_values: vals,
    })
}
