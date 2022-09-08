use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	set_dummy_benchmark {
	  // Benchmark setup phase
	  let b in 1 .. 1000;
	}: set_dummy(RawOrigin::Root, b.into()) // Execution phase
	verify {
		  // Optional verification phase
		let claim = vec![0, 1];
		let bounded_claim =
			<BoundedVec<u8, <T as Config>::MaxclaimLength>>::try_from(claim.clone()).unwrap();

		assert_eq!(
			<Proofs<T>>::get(&bounded_claim),
			Some((1, <frame_system::Pallet<T>>::block_number()))
		);
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	   );
   }  