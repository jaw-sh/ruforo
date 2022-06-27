#[test]
fn test_init_data() {
    use super::collection_values::CollectionValues;
    use super::flag::Flag;
    use super::mask::Mask;

    let mut cv = CollectionValues::default();

    {
        // set flags in a block so this data is dropped
        // just to make sure it is retained correctly.
        let values: Vec<(u8, u8, Flag)> = vec![
            (0, 0, Flag::YES),
            (0, 1, Flag::YES),
            (0, 2, Flag::YES),
            (0, 3, Flag::YES),
            (1, 0, Flag::YES),
            (2, 1, Flag::DEFAULT),
            (3, 2, Flag::NO),
            (4, 3, Flag::NEVER),
        ];

        for datum in values.iter() {
            cv.set_flag(datum.0, datum.1, datum.2);
        }
    }

    // test actual category values
    assert_eq!(cv.categories[0].yes, 15);
    assert_eq!(cv.categories[0].no, 0);
    assert_eq!(cv.categories[0].never, 0);
    assert_eq!(cv.categories[1].yes, 1);
    assert_eq!(cv.categories[2].yes, 0);
    assert_eq!(cv.categories[2].no, 0);
    assert_eq!(cv.categories[2].never, 0);
    assert_eq!(cv.categories[3].no, 4);
    assert_eq!(cv.categories[4].never, 8);

    // test bitmask operations
    let mask = Mask::from(cv);

    assert_eq!(mask.can(0, 0), true);
    assert_eq!(mask.can(0, 1), true);
    assert_eq!(mask.can(0, 2), true);
    assert_eq!(mask.can(0, 3), true);
    assert_eq!(mask.can(1, 0), true);
    assert_eq!(mask.can(1, 1), false);
    assert_eq!(mask.can(1, 2), false);
    assert_eq!(mask.can(1, 3), false);
}

#[test]
fn test_init_structure() {
    use super::collection::Collection;

    let mut col = Collection::default();

    {
        let item_data: Vec<(i32, i32, &str)> = vec![
            (11, 11, "a"),
            (53, 22, "b"),
            (59, 11, "c"),
            (12, 22, "d"),
            (70, 11, "e"),
            (99, 22, "f"),
            (16, 11, "g"),
            (22, 22, "h"),
        ];

        // Step 1. Pull unique categories.
        let mut ucid: Vec<&i32> = item_data.iter().map(|(_, b, _)| b).collect();
        ucid.sort_unstable();
        ucid.dedup();

        // Step 2. Add categories to collection.
        for (i, cid) in ucid.iter().enumerate() {
            col.categories[i].id = **cid;
            col.categories[i].position = i as u8;

            // Step 3. Add items to category.
            for item_datum in item_data.iter() {
                if item_datum.1 == col.categories[i].id {
                    match col.categories[i].add_item(item_datum.0, &item_datum.2) {
                        Ok(_) => {
                            println!("Added")
                        }
                        Err(_) => assert!(false, "Category overflow?"),
                    }
                }
            }
        }
    }

    // Step 4. Build the item name -> item pos tuple dictionary.
    col.build_dictionary();

    // test outside of block to ensure data survives
    assert_eq!(col.categories[0].id, 11);
    assert_eq!(col.categories[1].id, 22);
    assert_eq!(col.categories[3].id, 0);

    assert_eq!(col.categories[0].items[0].id, 11);
    assert_eq!(col.categories[0].items[0].category, 11);
    assert_eq!(col.categories[0].items[0].label, "a");
    assert_eq!(col.categories[0].items[0].position, 0);
    assert_eq!(col.categories[0].items[1].position, 1);
    assert_eq!(col.categories[0].items[2].position, 2);
    assert_eq!(col.categories[0].items[3].position, 3);
    assert_eq!(col.categories[1].items[3].label, "h");

    assert_eq!(col.categories[0].items[5].id, 0);
    assert_eq!(col.categories[2].items[0].id, 0);

    assert_eq!(col.get_item_pos("a").unwrap_or((9, 9)), (0, 0));
    assert_ne!(col.get_item_pos("b").unwrap_or((9, 9)), (0, 0));
    assert_eq!(col.get_item_pos("h").unwrap_or((9, 9)), (1, 3));
}

#[test]
fn test_mask_can() {
    use super::mask::Mask;

    let mut mask: Mask = Default::default();
    mask.categories[0] = 0b0101u64;

    assert_eq!(mask.can(0, 0), true);
    assert_eq!(mask.can(0, 1), false);
    assert_eq!(mask.can(0, 2), true);
    assert_eq!(mask.can(0, 3), false);
    assert_eq!(mask.can(0, 4), false);

    assert_eq!(mask.can(1, 0), false);
    assert_eq!(mask.can(2, 1), false);
    assert_eq!(mask.can(3, 2), false);
    assert_eq!(mask.can(4, 3), false);
}

#[test]
fn test_permission_add() {
    use super::category::Category;
    use super::PERM_LIMIT;
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut cat = Category::default();

    for i in 0..PERM_LIMIT {
        let id: i32 = rng.gen_range(1..999);
        let label: String = String::from(rng.gen_range(b'A'..b'Z') as char);
        match cat.add_item(id, &label) {
            Ok(p) => assert_eq!(p.position as u32, i),
            Err(_) => assert!(false, "Unexpected overflowing permission category"),
        }
    }

    let id = (PERM_LIMIT + 1) as i32;
    let label: &str = "foo";
    let pr = cat.add_item(id, label);
    assert!(pr.is_err(), "Did not contain an error.");
}

#[test]
fn test_set_join() {
    use super::category_values::CategoryValues;
    use super::collection_values::CollectionValues;

    let group1a = CategoryValues {
        yes: 0b0011u64,
        no: 0b0000u64,
        never: 0b0000u64,
    };
    let group1b = CategoryValues {
        yes: 0b0000u64,
        no: 0b1000u64,
        never: 0b0100u64,
    };
    let group2a = CategoryValues {
        yes: 0b0000u64,
        no: 0b0010u64,
        never: 0b0001u64,
    };
    let group2b = CategoryValues {
        yes: 0b1100u64,
        no: 0b0000u64,
        never: 0b0000u64,
    };

    let mut set1 = CollectionValues::default();
    set1.categories[0] = group1a;
    set1.categories[1] = group1b;

    let mut set2 = CollectionValues::default();
    set2.categories[0] = group2a;
    set2.categories[1] = group2b;

    let set3 = set1.join(&set2);

    assert_eq!(set1.categories[0].yes, 0b0011u64);
    assert_eq!(set2.categories[1].yes, 0b1100u64);
    assert_eq!(set3.categories[0].yes, 0b0010u64);
    assert_eq!(set3.categories[1].yes, 0b1000u64);
}

#[test]
fn test_values_join_combines_never() {
    use super::category_values::CategoryValues;

    let group1 = CategoryValues {
        yes: 0b100u64,
        no: 0b000u64,
        never: 0b010u64,
    };
    let group2 = CategoryValues {
        yes: 0b011u64,
        no: 0b000u64,
        never: 0b100u64,
    };
    let group3 = group1.join(&group2);

    assert_eq!(group3.yes, 0b0001u64);
    assert_eq!(group3.no, 0b0000u64);
    assert_eq!(group3.never, 0b0110u64);
}

#[test]
fn test_values_join_overwrites_no() {
    use super::category_values::CategoryValues;

    let group1 = CategoryValues {
        yes: 0b111u64,
        no: 0b000u64,
        never: 0b000u64,
    };
    let group2 = CategoryValues {
        yes: 0b0000u64,
        no: 0b0010u64,
        never: 0b1000u64,
    };
    let group3 = group1.join(&group2);

    assert_eq!(group3.yes, 0b0111u64);
    assert_eq!(group3.no, 0b0000u64);
    assert_eq!(group3.never, 0b1000u64);
}

#[test]
fn test_values_stack_negatives() {
    use super::category_values::CategoryValues;

    let group1 = CategoryValues {
        yes: 0b01100u64,
        no: 0b00010u64,
        never: 0b00001u64,
    };
    let group2 = CategoryValues {
        yes: 0b10011u64,
        no: 0b01111u64,
        never: 0b01000u64,
    };
    let group3 = group1.stack(&group2);

    assert_eq!(group3.yes, 0b10100u64);
    assert_eq!(group3.no, 0b00010u64);
    assert_eq!(group3.never, 0b01001u64);
}
