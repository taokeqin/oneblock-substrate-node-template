use crate::{mock::*, Error, Proofs};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_noop!(
			PoeModule::create_claim(RuntimeOrigin::signed(1), claim),
			Error::<Test>::ProofAlreadyExist
		);
	});
}

#[test]
fn revoke_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), None);
	});
}

#[test]
fn revoke_failed_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim),
			Error::<Test>::ClaimNotExist
		);
	});
}

#[test]
fn revoke_failed_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(2), claim),
			Error::<Test>::NotClaimOwner
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_failed_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim, 2),
			Error::<Test>::ClaimNotExist
		);
	});
}

#[test]
fn transfer_failed_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0; 10]).unwrap();
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(2), claim, 3),
			Error::<Test>::NotClaimOwner
		);
	});
}
