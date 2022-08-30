#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResultWithPostInfo, *},
		Blake2_128Concat, BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The maximum length of claim that can be added.
		#[pallet::constant]
		type MaxclaimLength: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxclaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfer(T::AccountId, Vec<u8>, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = <BoundedVec<u8, T::MaxclaimLength>>::try_from(claim.clone())
				.map_err(|_| <Error<T>>::ClaimTooLong)?;

			ensure!(!<Proofs<T>>::contains_key(&bounded_claim), <Error<T>>::ProofAlreadyExist);

			<Proofs<T>>::insert(
				&bounded_claim,
				(&sender, <frame_system::Pallet<T>>::block_number()),
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = Self::check(&sender, &claim)?;

			<Proofs<T>>::remove(&bounded_claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = Self::check(&sender, &claim)?;

			<Proofs<T>>::insert(&bounded_claim, (&dest, <frame_system::Pallet<T>>::block_number()));

			Self::deposit_event(Event::ClaimTransfer(sender, claim, dest));

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn check(
			sender: &T::AccountId,
			claim: &Vec<u8>,
		) -> Result<BoundedVec<u8, T::MaxclaimLength>, Error<T>> {
			let bounded_claim = <BoundedVec<u8, T::MaxclaimLength>>::try_from(claim.clone())
				.map_err(|_| <Error<T>>::ClaimTooLong)?;

			let owner = <Proofs<T>>::get(&bounded_claim).ok_or(<Error<T>>::ClaimNotExist)?.0;

			ensure!(owner == *sender, <Error<T>>::NotClaimOwner);

			Ok(bounded_claim)
		}
	}
}
