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

pub use category::Category;
pub use category_values::CategoryValues;
pub use flag::Flag;
pub use item::Item;

// Values sort like this:
// Item:       Y/N
// Category:   {Yes,No,Never} * u64[array of Item flags]
// Collection: u32[array of Category values]

/// Maximum number of permission categories
pub const GROUP_LIMIT: u32 = 16;
/// Maximum number of permissions per category (64 bits)
pub const PERM_LIMIT: u32 = u64::BITS;
/// Total maximum number of permissions defined as GROUP_LIMIT*PERM_LIMIT
pub const MAX_PERMS: u32 = GROUP_LIMIT * PERM_LIMIT;

use crate::middleware::ClientCtx;
use dashmap::DashMap;

#[derive(Clone, Debug, Default)]
pub struct PermissionData {
    /// Threadsafe Data Structure
    collection: collection::Collection,
    /// (Group, User) -> CollectionValues Relationship
    collection_values: DashMap<(i32, i32), collection_values::CollectionValues>,
}

impl PermissionData {
    /// Accepts Client/Guest and Permission Name for permission check.
    pub fn can(&self, client: &ClientCtx, permission: &str) -> bool {
        // Look up the permissions's indices by name.
        if let Some(pindices) = self.collection.dictionary.get(permission) {
            self.can_by_indices(client, &pindices)
        } else {
            log::warn!(
                "Bad permission check on name '{:?}', which is not present in our dictionary.",
                permission
            );
            false
        }
    }

    /// Accepts Client/Guest and Permission ID for permission check.
    pub fn can_by_id(&self, client: &ClientCtx, permission_id: i32) -> bool {
        // Look up the permissions's indices by id.
        if let Some(pindices) = self.collection.lookup.get(&permission_id) {
            self.can_by_indices(client, &pindices)
        } else {
            log::warn!(
                "Bad permission check on id {:?}, which is not present in our dictionary.",
                permission_id
            );
            false
        }
    }

    /// Accepts Client/Guest and specific permission indices for permission check.
    pub fn can_by_indices(&self, client: &ClientCtx, indices: &(u8, u8)) -> bool {
        let groups = client.get_groups();
        let values = match client.get_id() {
            Some(id) => {
                let group_values = self.join_for_groups(&groups);
                let user_values = self.join_for_user(id);
                group_values.join(&user_values)
            }
            None => self.join_for_groups(&groups),
        };

        let mask = mask::Mask::from(values);
        mask.can(indices.0 as usize, indices.1 as i32)
    }

    pub fn join_for_groups(&self, groups: &Vec<i32>) -> collection_values::CollectionValues {
        use collection_values::CollectionValues;
        let mut return_values = CollectionValues::default();

        for group in groups {
            let val_key = (group.to_owned(), 0);

            if let Some(group_values) = self.collection_values.get(&val_key) {
                return_values = return_values.join(&group_values);
            }
        }

        return_values
    }

    pub fn join_for_user(&self, id: i32) -> collection_values::CollectionValues {
        use collection_values::CollectionValues;
        let mut return_values = CollectionValues::default();
        let val_key = (0, id);

        if let Some(group_values) = self.collection_values.get(&val_key) {
            return_values = return_values.join(&group_values);
        }

        return_values
    }
}

pub async fn new() -> Result<PermissionData, sea_orm::error::DbErr> {
    use crate::db::get_db_pool;
    use crate::orm::permission_collections;
    use crate::orm::permission_values;
    use crate::orm::permissions;
    use collection_values::CollectionValues;
    use sea_orm::entity::*;

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
                match col.categories[i].add_item(item.id, &item.label) {
                    Ok(item) => {
                        col.dictionary
                            .insert(item.label.to_owned(), (i as u8, item.position as u8));
                        col.lookup.insert(item.id, (i as u8, item.position as u8));
                    }
                    Err(_) => {
                        println!("Category overflow adding {:?}", item);
                    }
                }
            }
        }
    }

    // Import data
    let vals: DashMap<(i32, i32), CollectionValues> = Default::default();
    let perm_collections = permission_collections::Entity::find()
        .find_with_related(permission_values::Entity)
        .all(get_db_pool())
        .await?;

    // convert ORM data into permission system structs
    // loop through the collection-<values relations
    for (perm_collection, pvs) in perm_collections.iter() {
        // Create collection values record to set flags on
        let mut cv = CollectionValues::default();

        // loop through the values
        for pv in pvs.iter() {
            // Look up the permissions's indices by id.
            if let Some(pindices) = col.lookup.get(&pv.permission_id) {
                // Assign each flag to the CollectionValues.
                cv.set_flag(pindices.0, pindices.1, pv.value);
            } else {
                println!(
                    "Failed to lookup indices for permission_values {:?},{:?}",
                    pv.collection_id, pv.permission_id
                );
            }
        }

        // Resolve (group,user) tuple key
        let val_key: (i32, i32) = (
            perm_collection.group_id.unwrap_or(0),
            perm_collection.user_id.unwrap_or(0),
        );

        if vals.contains_key(&val_key) {
            // Join permission with same key.
            vals.alter(&val_key, |_, v| cv.join(&v));
        } else {
            // Add to values lookup.
            vals.insert(val_key, cv);
        }
    }

    Ok(PermissionData {
        collection: col,
        collection_values: vals,
    })
}
