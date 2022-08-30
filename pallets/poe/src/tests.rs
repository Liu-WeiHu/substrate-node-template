use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			<BoundedVec<u8, <Test as Config>::MaxclaimLength>>::try_from(claim.clone()).unwrap();

		assert_eq!(
			<Proofs<Test>>::get(&bounded_claim),
			Some((1, <frame_system::Pallet<Test>>::block_number()))
		);

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			<Error<Test>>::ProofAlreadyExist
		);

		let claim = vec![1; 513];

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim),
			<Error<Test>>::ClaimTooLong
		);
	})
}

#[test]
fn remove_claim() {
	new_test_ext().execute_with(|| {
		let claim = vec![1; 511];

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim),
			<Error<Test>>::ClaimNotExist
		);
	})
}

#[test]
fn transfer_claim() {
	new_test_ext().execute_with(|| {
		let claim = vec![1; 511];

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
			<Error<Test>>::NotClaimOwner
		);

		let bounded_claim =
			<BoundedVec<u8, <Test as Config>::MaxclaimLength>>::try_from(claim).unwrap();

		assert_eq!(
			<Proofs<Test>>::get(&bounded_claim),
			Some((2, <frame_system::Pallet<Test>>::block_number()))
		);
	})
}
