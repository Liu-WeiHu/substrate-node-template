use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use crate::*;

benchmarks! {
   create_claim {
      let d in 0 .. T::MaxclaimLength::get();
      let claim = vec![0; d as usize];
      let caller: T::AccountId = whitelisted_caller();
   }: _(RawOrigin::Signed(caller.clone()), claim.clone())
   	verify {
        let bounded_claim = <BoundedVec<u8, T::MaxclaimLength>>::try_from(claim).unwrap();
		assert_eq!(Proofs::<T>::get(&bounded_claim), Some((caller, <frame_system::Pallet<T>>::block_number())));
	}

   revoke_claim {
      let d in 0 .. T::MaxclaimLength::get();
      let claim = vec![0; d as usize];
      let caller: T::AccountId = whitelisted_caller();
      let bounded_claim = <BoundedVec<u8, T::MaxclaimLength>>::try_from(claim.clone()).unwrap();
      <Proofs<T>>::insert(&bounded_claim, (&caller, <frame_system::Pallet<T>>::block_number()));
   }: _(RawOrigin::Signed(caller), claim)
    verify {
        assert_eq!(Proofs::<T>::contains_key(&bounded_claim), false);
	}

   transfer_claim {
      let d in 0 .. T::MaxclaimLength::get();
      let claim = vec![0; d as usize];
      let caller: T::AccountId = whitelisted_caller();
      let caller2: T::AccountId = whitelisted_caller();
      let bounded_claim = <BoundedVec<u8, T::MaxclaimLength>>::try_from(claim.clone()).unwrap();
      <Proofs<T>>::insert(&bounded_claim, (&caller, <frame_system::Pallet<T>>::block_number()));
   }: _(RawOrigin::Signed(caller), claim, caller2.clone())
    verify {
		assert_eq!(Proofs::<T>::get(&bounded_claim), Some((caller2, <frame_system::Pallet<T>>::block_number())));
	}

   	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);

}