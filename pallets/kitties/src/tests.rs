use crate::{mock::*, Error, Event, Kitty, KittyId, NextKittyId};
use frame_support::{assert_noop, assert_ok};
//import testevent
use frame_system::{EventRecord, Phase};

#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
		assert_eq!(
			KittiesModule::kitties(kitty_id),
			Some(Kitty([215, 75, 66, 60, 234, 156, 146, 62, 247, 65, 230, 205, 192, 2, 31, 70])) // this value copy from a failed test, as it generated from the fixed block number.
		);
	});
}

// test event
#[test]
fn create_kitty_works_with_event_sent() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let who = 1;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
		let kitty = KittiesModule::kitties(kitty_id).unwrap();
		let expected_event = Event::KittyCreated { who, kitty_id, kitty };
		System::assert_has_event(expected_event.clone().into())
	});
}

#[test]
fn create_kitty_failed_when_kitty_count_exceeds_max_value() {
	new_test_ext().execute_with(|| {
		NextKittyId::<Test>::set(KittyId::max_value());
		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(1)),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = 0;
		let kitty_id_2 = 1;
		let kitty_id_3 = 2;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));
		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(1), kitty_id_1, kitty_id_2));
		assert_eq!(
			KittiesModule::kitties(2),
			Some(Kitty([
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
			]))
		);
		// check parent
		assert_eq!(KittiesModule::kitty_parents(kitty_id_3), (kitty_id_1, kitty_id_2));
	});
}

// test event KittyBreeded
#[test]
fn breed_kitty_works_with_event_sent() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = 0;
		let kitty_id_2 = 1;
		let kitty_id_3 = 2;
		let who = 1;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(who)));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(who)));
		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(who), kitty_id_1, kitty_id_2));
		let kitty = KittiesModule::kitties(kitty_id_3).unwrap();
		let expected_event = Event::KittyBreed { who, kitty_id: kitty_id_3, kitty };
		System::assert_has_event(expected_event.clone().into())
	});
}

#[test]
fn breed_kitty_failed_when_kitty_count_exceeds_max_value() {
	new_test_ext().execute_with(|| {
		let kitty_id_1 = 0;
		let kitty_id_2 = 1;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1))); // 0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1))); // 1
		NextKittyId::<Test>::set(KittyId::max_value());
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(1), kitty_id_1, kitty_id_2),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let from_account_id = 1;
		let to_account_id = 2;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(from_account_id)));
		assert_ok!(KittiesModule::transfer(
			RuntimeOrigin::signed(from_account_id),
			to_account_id,
			kitty_id
		));
		assert_eq!(KittiesModule::owner(kitty_id), Some(2));
	});
}

// test event KittyTransferred
#[test]
fn transfer_kitty_works_with_event_sent() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let from_account_id = 1;
		let to_account_id = 2;
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(from_account_id)));
		assert_ok!(KittiesModule::transfer(
			RuntimeOrigin::signed(from_account_id),
			to_account_id,
			kitty_id
		));
		let expected_event =
			Event::KittyTransferred { from: from_account_id, to: to_account_id, kitty_id };
		System::assert_has_event(expected_event.clone().into())
	});
}
