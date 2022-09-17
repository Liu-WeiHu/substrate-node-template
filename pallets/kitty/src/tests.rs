use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, assert_err};

#[test]
fn test_create_kitty() {
	new_test_ext().execute_with(|| {
		// 创建两个kitty
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
        run_to_block(2);
		assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
        // 再创建kitty失败,因为质押财产不够.
        run_to_block(3);
        assert_noop!(
            KittyModule::create_kitty(Origin::signed(1)),
            <Error<Test>>::ReserveBalanceFailed
        );

        //创建三个kitty
        assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
        run_to_block(4);
        assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
        run_to_block(5);
        assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
        run_to_block(6);
        // 再创建kitty失败,因为超过账户最大持有量.
        assert_err!(KittyModule::create_kitty(Origin::signed(2)), <Error<Test>>::ExceedMaxKittyOwned);
	});
}

#[test]
fn test_set_price() {
    new_test_ext().execute_with(|| {
        // 创建中账户1 的kitty
        KittyModule::create_kitty(Origin::signed(1)).unwrap();
        // kitty总数为1
        assert_eq!(super::pallet::CountKitties::<Test>::get(), 1);
        let kitty_id = super::pallet::KittiesOwned::<Test>::get(1)[0];
        // 判断kitty  不属于 账户2
        assert_err!(KittyModule::set_price(Origin::signed(2), kitty_id, Some(100)), <Error<Test>>::NotKittyOwner);
        //  给kitty设置价格
        assert_ok!(KittyModule::set_price(Origin::signed(1), kitty_id, Some(100)));
        // 判断kitty的价格是不是等于100
        let kitty = super::pallet::Kitties::<Test>::get(&kitty_id).unwrap();
        assert_eq!(kitty.price, Some(100));
    })
}

#[test]
fn test_transfer() {
    new_test_ext().execute_with(|| {
        // 创建中账户1 的kitty
        KittyModule::create_kitty(Origin::signed(1)).unwrap();
        // 获取kitty_id
        let kitty_id = super::pallet::KittiesOwned::<Test>::get(1)[0];
        // 发起失败交易
        assert_err!(KittyModule::transfer(Origin::signed(1), kitty_id, 1), <Error<Test>>::TransferToSelf);
        // 发起失败交易
        assert_err!(KittyModule::transfer(Origin::signed(2), kitty_id, 1), <Error<Test>>::NotKittyOwner);
        // 成功交易
        assert_ok!(KittyModule::transfer(Origin::signed(1), kitty_id, 2));
        // 获取账户2的kitty 判断是否属于账户2
        let kitty_id = super::pallet::KittiesOwned::<Test>::get(2)[0];
        assert_eq!(super::pallet::Kitties::<Test>::get(&kitty_id).unwrap().owner, 2);
    })
}

#[test]
fn test_buy_kitty() {
    new_test_ext().execute_with(|| {
        // 创建中账户1 的kitty
        KittyModule::create_kitty(Origin::signed(2)).unwrap();
        // 获取kitty_id
        let kitty_id = super::pallet::KittiesOwned::<Test>::get(2)[0];
        // 设置kitty的价格
        assert_ok!(KittyModule::set_price(Origin::signed(2), kitty_id, Some(90)));
        // 不能同一个账户购买
        assert_err!(KittyModule::buy_kitty(Origin::signed(2), kitty_id, 300), <Error<Test>>::BuyerIsKittyOwner);
        // 购买价低于售价
        assert_err!(KittyModule::buy_kitty(Origin::signed(1), kitty_id, 50), <Error<Test>>::KittyBidPriceTooLow);
        // 余额不足
        assert_err!(KittyModule::buy_kitty(Origin::signed(1), kitty_id, 300), <Error<Test>>::NotEnoughBalance);
        // 没有余额质押资产
        assert_err!(KittyModule::buy_kitty(Origin::signed(1), kitty_id, 120), <Error<Test>>::ReserveBalanceFailed);
        // 成功购买
        assert_ok!(KittyModule::buy_kitty(Origin::signed(1), kitty_id, 100));
        // 校验所有权
        let kitty_id = super::pallet::KittiesOwned::<Test>::get(1)[0];
        assert_eq!(super::pallet::Kitties::<Test>::get(&kitty_id).unwrap().owner, 1);
    })
}

#[test]
fn test_breed_kitty() {
    new_test_ext().execute_with(|| {
        // 创建kitty
        assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
        let kitty_id_owner2_1 = super::pallet::KittiesOwned::<Test>::get(2)[0];
        assert_ok!(KittyModule::create_kitty(Origin::signed(1)));
        let kitty_id_owner1_1 = super::pallet::KittiesOwned::<Test>::get(1)[0];
        run_to_block(2);
        assert_ok!(KittyModule::create_kitty(Origin::signed(2)));
        let kitty_id_owner2_2 = super::pallet::KittiesOwned::<Test>::get(2)[1];
        // 校验所有权
        assert_err!(KittyModule::breed_kitty(Origin::signed(2), kitty_id_owner1_1, kitty_id_owner2_1), <Error<Test>>::NotKittyOwner);
        // 成功繁殖
        assert_ok!(KittyModule::breed_kitty(Origin::signed(2), kitty_id_owner2_1, kitty_id_owner2_2));
        // 检查新生kitty 是否属于账户2
        let kitty_id_owner2_3 = super::pallet::KittiesOwned::<Test>::get(2)[2];
        assert_eq!(super::pallet::Kitties::<Test>::get(&kitty_id_owner2_3).unwrap().owner, 2);
    })
}