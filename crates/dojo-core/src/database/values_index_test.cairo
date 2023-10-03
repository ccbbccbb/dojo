use array::ArrayTrait;
use traits::Into;
use debug::PrintTrait;
use option::OptionTrait;


use dojo::database::values_index;

#[test]
#[available_gas(2000000)]
fn test_index_same_values() {
    let no_query = values_index::query(0, 69, 1);
    assert(no_query.len() == 0, 'entity indexed');

    values_index::create(0, 69, 420, 1);
    let query = values_index::query(0, 69, 1);
    assert(query.len() == 1, 'entity not indexed');
    assert(*query.at(0) == 420, 'entity value incorrect');

    values_index::create(0, 69, 420, 1);
    let noop_query = values_index::query(0, 69, 1);
    assert(noop_query.len() == 1, 'index should be noop');

    values_index::create(0, 69, 1337, 1);
    let two_query = values_index::query(0, 69, 1);
    assert(two_query.len() == 2, 'index should have two query');
    assert(*two_query.at(1) == 1337, 'entity value incorrect');
}

#[test]
#[available_gas(2000000)]
fn test_index_different_values() {
    values_index::create(0, 69, 420, 1);
    let query = values_index::query(0, 69, 1);
    assert(query.len() == 1, 'entity not indexed');
    assert(*query.at(0) == 420, 'entity value incorrect');

    let noop_query = values_index::query(0, 69, 3);
    assert(noop_query.len() == 0, 'index should be noop');

    values_index::create(0, 69, 1337, 2);
    values_index::create(0, 69, 1337, 2);
    values_index::create(0, 69, 1338, 2);
    let two_query = values_index::query(0, 69, 2);
    assert(two_query.len() == 2, 'index should have two query');
    assert(*two_query.at(1) == 1338, 'two query value incorrect');
}

// #[test]
// #[available_gas(100000000)]
// fn test_entity_delete_basic() {
//     values_index::create(0, 69, 420, 1);
//     let query = values_index::query(0, 69, 1);
//     assert(query.len() == 1, 'entity not indexed');
//     assert(*query.at(0) == 420, 'entity value incorrect');

//     assert(values_index::exists(0, 69, 420), 'entity should exist');

//     values_index::delete(0, 69, 420);

//     assert(!values_index::exists(0, 69, 420), 'entity should not exist');
//     let no_query = values_index::query(0, 69, 1);
//     assert(no_query.len() == 0, 'index should have no query');
// }

// #[test]
// #[available_gas(100000000)]
// fn test_entity_query_delete_shuffle() {
//     let table = 1;
//     values_index::create(0, table, 10, 1);
//     values_index::create(0, table, 20, 1);
//     values_index::create(0, table, 30, 1);
//     assert(values_index::query(0, table, 1).len() == 3, 'wrong size');

//     values_index::delete(0, table, 10);
//     let entities = values_index::query(0, table, 1);
//     assert(entities.len() == 2, 'wrong size');
//     assert(*entities.at(0) == 30, 'idx 0 not 30');
//     assert(*entities.at(1) == 20, 'idx 1 not 20');
// }

// #[test]
// #[available_gas(100000000)]
// fn test_entity_query_delete_non_existing() {
//     assert(values_index::query(0, 69, 1).len() == 0, 'table len != 0');
//     values_index::delete(0, 69, 999); // deleting non-existing should not panic
// }
