#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{transactional, pallet_prelude::{*, ValueQuery, DispatchResultWithPostInfo}, traits::{Currency, Randomness, ExistenceRequirement, ReservableCurrency}, Twox64Concat, BoundedVec, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_io::hashing::blake2_128;

    // 定义账户类型
    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    // 定义资产类型
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountOf<T>>>::Balance;
    // 定义质押资产类型
    type ReserveBalanceOf<T> = <<T as Config>::ReservableCurrency as Currency<AccountOf<T>>>::Balance;

    #[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    // 定义kitty结构体 包含 dna、价格、性别、所有者 元素
    pub struct Kitty<T: Config> {
        pub dna: [u8; 16],
        pub price: Option<BalanceOf<T>>,
        pub gender: Gender,
        pub owner: AccountOf<T>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, TypeInfo)]
    // 定义性别枚举体
    pub enum Gender {
        Male,
        Female,
    }

    #[pallet::config]
	pub trait Config: frame_system::Config {
        // 事件
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // 资产
        type Currency: Currency<Self::AccountId>;
        // 随机性
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        #[pallet::constant]
        // 持有kitty的最大数量限制
        type MaxKittyOwned: Get<u32>;
        // 质押资产
		type ReservableCurrency: ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
        // 质押费用
		type ReservationFee: Get<ReserveBalanceOf<Self>>;
    }

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
    // kitty 统计总数 最多 无符号64位
    pub type CountKitties<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    // kitty id对应kitty结构体
    pub type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

    #[pallet::storage]
    // 所有者账户 对应 kitty id集合。集合有MaxKittyOwned限制长度
    pub type KittiesOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittyOwned>, ValueQuery>;

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        // 成功创建kitty
        Created(T::AccountId, T::Hash),
        // 为kitty设置价格
        PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
        // 交易成功
        Transferred(T::AccountId, T::AccountId, T::Hash),
        // 成功购买
        Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
    }

    #[pallet::error]
	pub enum Error<T> {
        // kitty 增加时溢出
        CountForKittiesOverflow,
        // 账户不能超过最大拥有量
        ExceedMaxKittyOwned,
        // 买家不能成为所有者
        BuyerIsKittyOwner,
        // 无法交易
        TransferToSelf,
        // kitty 已经存在
        KittyExists,
        // kitty 不存在
        KittyNotExist,
        // 这只 kitty 不是该所有者的
        NotKittyOwner,
        // 确保 kitty 可以售卖
        KittyNotForSale,
        // 确保购买价格大于要价。
        KittyBidPriceTooLow,
        // 确保账户拥有足够的 balance
        NotEnoughBalance,
        // 质押资产失败
        ReserveBalanceFailed,
    }

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
	impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        // 创建 kitty 
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // 调用生成kitty 方法
            let kitty_id = Self::mint(&sender, None, None)?;
            Self::deposit_event(Event::Created(sender, kitty_id));
            Ok(().into())
        }

        #[pallet::weight(100)]
        // 设置 kitty 价格
        pub fn set_price(origin: OriginFor<T>, kitty_id: T::Hash, price: Option<BalanceOf<T>>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // 确保 kitty id 存在，并且归所有者 所有
            ensure!(Self::is_kitty_owner(&sender, &kitty_id)?, <Error<T>>::NotKittyOwner);
            // 确保 kitty 存在
            let mut kitty = <Kitties<T>>::get(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
            // 设置价格
            kitty.price = price;
            // 重新插入
            <Kitties<T>>::insert(&kitty_id, kitty);
            Self::deposit_event(Event::PriceSet(sender, kitty_id, price));
            Ok(().into())
        }

        #[pallet::weight(100)]
        // 交易 kitty
        pub fn transfer(origin: OriginFor<T>, kitty_id: T::Hash, to: T::AccountId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // 确保 kitty id 存在，并且归所有者 所有
            ensure!(Self::is_kitty_owner(&sender, &kitty_id)?, <Error<T>>::NotKittyOwner);
            // 确保 交易双方非同一个账户
            ensure!(sender != to, <Error<T>>::TransferToSelf);
            // 开始交易
            Self::transfer_kitty_to(&kitty_id, &to)?;
            Self::deposit_event(Event::Transferred(sender, to, kitty_id));
            Ok(().into())
        }

        #[transactional]
		#[pallet::weight(100)]
        // 买入 kitty
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			bid_price: BalanceOf<T>
		) -> DispatchResultWithPostInfo {
			let buyer = ensure_signed(origin)?;

            // 确保kitty存在
			let kitty = <Kitties<T>>::get(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
            // 确保交易双方不是同一账户
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

            // 确保kitty 设置了价格并且价格低于 售卖价格
			if let Some(ask_price) = kitty.price {
                ensure!(ask_price < bid_price, <Error<T>>::KittyBidPriceTooLow);
            } else {
                Err(<Error<T>>::KittyNotForSale)?;
            }

            // 确保买方 余额 大于等于 售卖价格
            ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);
            // 交易金额
            T::Currency::transfer(&buyer, &kitty.owner.clone(), bid_price, ExistenceRequirement::KeepAlive)?;
            let seller = kitty.owner;
            // 交易kitty
            Self::transfer_kitty_to(&kitty_id, &buyer)?;
            Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));
			Ok(().into())
		}

        #[pallet::weight(100)]
        pub fn breed_kitty(origin: OriginFor<T>, parent1: T::Hash, parent2: T::Hash) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // 确保我有这两个kitty
            ensure!(Self::is_kitty_owner(&sender, &parent1)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&sender, &parent2)?, <Error<T>>::NotKittyOwner);
            // 繁殖出 dna
            let new_dna = Self::breed_dna(&parent1, &parent2)?;
            // 制造出 kitty
			Self::mint(&sender, Some(new_dna), None)?;
            Ok(().into())
        }

    }

    impl<T: Config> Pallet<T> {
        // 生成性别
        fn gen_gender() -> Gender {
            let random = T::Randomness::random(b"gen_gender").0;
            match random.as_ref()[0] & 1 {
                0 => Gender::Male,
		        _ => Gender::Female,
            }
        }

        // 生成 dna
        fn gen_dna() -> [u8; 16] {
            let payload = (
                T::Randomness::random(b"gen_dna").0,
                <frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
                <frame_system::Pallet<T>>::block_number(),
            );
            payload.using_encoded(blake2_128)
        }

        // 制造 kitty
        fn mint(owner: &T::AccountId, dna: Option<[u8; 16]>, gender: Option<Gender>) -> Result<T::Hash, Error<T>> {

            // 初始化kitty 结构体
            let kitty = Kitty::<T>{
                dna: dna.unwrap_or_else(Self::gen_dna),
                price: None,
                gender: gender.unwrap_or_else(Self::gen_gender),
                owner: owner.clone(),
            };
            // hash散列 生成kitty id
            let kitty_id = T::Hashing::hash_of(&kitty);
            // 质押资产
            let deposit = T::ReservationFee::get();
            T::ReservableCurrency::reserve(owner, deposit).map_err(|_|<Error<T>>::ReserveBalanceFailed)?;
            // 检验是否超过最大值
            let count = <CountKitties<T>>::get().checked_add(1).ok_or(<Error<T>>::CountForKittiesOverflow)?;
            // 确保没有相同的kitty id 存在
            ensure!(<Kitties<T>>::get(&kitty_id) == None, <Error<T>>::KittyExists);
            // 确保 所有者的 kitty 数量没有超过最大值
            <KittiesOwned<T>>::try_mutate(owner, |v| {
                v.try_push(kitty_id)
            }).map_err(|_|<Error<T>>::ExceedMaxKittyOwned)?;
            // 插入
            <Kitties<T>>::insert(kitty_id, kitty);
            <CountKitties<T>>::put(count);
            Ok(kitty_id)
        }

        // 判断 kitty id 是非存在 
        // 若存在 并校验 是非归所有者 所有
        fn is_kitty_owner(owner: &T::AccountId, kitty_id: &T::Hash) -> Result<bool, Error<T>> {
            match <Kitties<T>>::get(kitty_id) {
                Some(kitty) => Ok(kitty.owner == *owner),
                None => Err(<Error<T>>::KittyNotExist),
            }
        }

        #[transactional]
        // 交易给账户
        fn transfer_kitty_to(kitty_id: &T::Hash, to: &T::AccountId) -> DispatchResultWithPostInfo {
            // 确保 kitty 存在
            let mut kitty = <Kitties<T>>::get(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

            // 质押转移
            let deposit = T::ReservationFee::get();
            T::ReservableCurrency::reserve(to, deposit).map_err(|_|<Error<T>>::ReserveBalanceFailed)?;
            _ = T::ReservableCurrency::unreserve(&kitty.owner, deposit);

            // 确保 持有者的kitty 数量减少
            <KittiesOwned<T>>::try_mutate(kitty.owner.clone(), |v| {
                match v.iter().position(|&id| id == *kitty_id) {
                    Some(index) => {
                        v.swap_remove(index); 
                        Ok(().into())
                    }
                    None => Err(<Error<T>>::KittyNotExist),
                }
            })?;

            // 确保 交易方 kitty 数量增加
            <KittiesOwned<T>>::try_mutate(to, |v| {
                v.try_push(*kitty_id)
            }).map_err(|_|<Error<T>>::ExceedMaxKittyOwned)?;

            kitty.owner = to.clone();
            kitty.price = None;

            // 插入kitty
            <Kitties<T>>::insert(&kitty_id, kitty);
            Ok(().into())
        }

        // 繁殖 dna
        pub fn breed_dna(parent1: &T::Hash, parent2: &T::Hash) -> Result<[u8; 16], Error<T>> {
			let dna1 = <Kitties<T>>::get(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = <Kitties<T>>::get(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}
    }
}