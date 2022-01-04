#![cfg(test)]

use crate::{
	mock::*, pallet::{Error}
};
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(3)));
	});
}

#[test]
fn create_kitty_failed_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KittiesModule::create(Origin::signed(1)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_kitty_trasfer_works() {
	new_test_ext().execute_with(|| {
		let to = 2;
		let kitty_id = 1;
		assert_ok!(KittiesModule::transfer(Origin::signed(1), to, kitty_id));
		let new_owner = KittiesModule::owner(kitty_id);
		assert_eq!(new_owner, Some(to));
	});
}

#[test]
fn create_kitty_trasfer_failed_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		let to = 2;
		let kitty_id = 10;
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), to, kitty_id),
			Error::<Test>::KittyNotExist
		);
	});
}

#[test]
fn create_kitty_trasfer_failed_kitty_not_onwer() {
	new_test_ext().execute_with(|| {
		let to = 2;
		let kitty_id = 1;
		assert_noop!(
			KittiesModule::transfer(Origin::signed(3), to, kitty_id),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn set_price_works() {
	new_test_ext().execute_with(|| {		
		let kitty_id = 1;
		let new_price = 100;
		assert_ok!(KittiesModule::set_price(Origin::signed(1), kitty_id, Some(new_price)));
		let kitty = KittiesModule::kitties(kitty_id).expect("should found the kitty");
		assert_eq!(Some(new_price), kitty.price);
	});
}


#[test]
fn set_price_failed_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		let kitty_id = 10;
		let new_price = 100;

		assert_noop!(
			KittiesModule::set_price(Origin::signed(1), kitty_id, Some(new_price)),
			Error::<Test>::KittyNotExist
		);
	});
}

#[test]
fn set_price_failed_kitty_not_onwer() {
	new_test_ext().execute_with(|| {
		let kitty_id = 2;
		let new_price = 100;

		assert_noop!(
			KittiesModule::set_price(Origin::signed(1), kitty_id, Some(new_price)),
			Error::<Test>::NotOwner
		);
	});
}



#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {		
		let kitty_id = 1;
		let price = 100;
		KittiesModule::set_price(Origin::signed(1), kitty_id, Some(price)).expect("should set the price");

		assert_ok!(KittiesModule::buy(Origin::signed(3), kitty_id, price));
		let owner = KittiesModule::owner(kitty_id).expect("should found the kitty");
		assert_eq!(3, owner);
	});
}


#[test]
fn buy_failed_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		let kitty_id = 10;
		let price = 100;
		assert_noop!(
			KittiesModule::buy(Origin::signed(3), kitty_id, price),
			Error::<Test>::KittyNotExist
		);
	});
}


#[test]
fn buy_failed_owner_can_not_buy() {
	new_test_ext().execute_with(|| {
		let kitty_id = 1;
		let price = 5;
		assert_noop!(
			KittiesModule::buy(Origin::signed(1), kitty_id, price),
			Error::<Test>::OwnerCanNotBuy
		);
	});
}

#[test]
fn set_price_failed_price_too_low() {
	new_test_ext().execute_with(|| {
		let kitty_id = 1;
		KittiesModule::set_price(Origin::signed(1), kitty_id, Some(100)).expect("should set the price");

		assert_noop!(
			KittiesModule::buy(Origin::signed(3), kitty_id, 90),
			Error::<Test>::KittyBidPriceTooLow
		);
	});
}

#[test]
fn set_price_failed_kitty_not_for_sale() {
	new_test_ext().execute_with(|| {
		let kitty_id = 1;
		assert_noop!(
			KittiesModule::buy(Origin::signed(3), kitty_id, 90),
			Error::<Test>::KittyNotForSale
		);
	});
}