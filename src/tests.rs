use hex_literal::hex;
use support::{assert_err, assert_ok};

use crate::mock::{new_test_ext, Htlc, Origin, Test, Timestamp};
use crate::{HtlcState, Trait};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DAVE: u64 = 4;

const SYMBOL: [u8; 3] = [0x42, 0x54, 0x43];

#[test]
fn create_token_should_work() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let now = <Test as Trait>::Time::now();
		assert_eq!(now, 0);
		assert_eq!(Htlc::next_asset_id(), id);
		assert_ok!(Htlc::create_token(Origin::signed(ALICE), SYMBOL.to_vec()));
		assert_eq!(Htlc::total_supply(&id), 0);
		assert_eq!(Htlc::get_balance_of(&id, &ALICE), 0);
	});
}

#[test]
fn htlc_claim_should_work() {
	let id = 0;
	let amount = 1000;
	let secret_hash = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
	let now = 1000;
	let expiration = now + 1000;

	new_test_ext().execute_with(|| {
		Timestamp::set_timestamp(now);

		assert_ok!(Htlc::create_token(Origin::signed(ALICE), SYMBOL.to_vec()));

		assert_ok!(Htlc::create_htlc(
			Origin::signed(ALICE),
			SYMBOL.to_vec(),
			BOB,
			amount,
			secret_hash,
			expiration
		));

		let secret = vec![];
		assert_ok!(Htlc::claim(Origin::signed(ALICE), secret.clone()));
		assert_eq!(Htlc::get_balance_of(&id, &BOB), amount);
		assert_eq!(Htlc::total_supply(&id), amount);
		assert_err!(
			Htlc::claim(Origin::signed(ALICE), secret),
			"invalid htlc state"
		);
		assert_eq!(
			Htlc::htlc_of(secret_hash).unwrap().state,
			HtlcState::Claimed
		)
	});

	new_test_ext().execute_with(|| {
		Timestamp::set_timestamp(1000);

		let now = <Test as Trait>::Time::now();
		assert_eq!(now, 1000);

		assert_ok!(Htlc::create_token(Origin::signed(ALICE), SYMBOL.to_vec()));

		let expiration = now + 1000;

		assert_ok!(Htlc::create_htlc(
			Origin::signed(ALICE),
			SYMBOL.to_vec(),
			BOB,
			amount,
			secret_hash,
			expiration
		));

		let secret = vec![];
		assert_ok!(Htlc::claim(Origin::signed(ALICE), secret));

		assert_eq!(Htlc::get_balance_of(&id, &BOB), amount);
		assert_eq!(Htlc::get_balance_of(&id, &CHARLIE), 0);

		assert_ok!(Htlc::transfer(
			Origin::signed(BOB),
			SYMBOL.to_vec(),
			CHARLIE,
			amount
		));

		assert_eq!(Htlc::get_balance_of(&id, &BOB), 0);
		assert_eq!(Htlc::get_balance_of(&id, &CHARLIE), amount);

		assert_err!(
			Htlc::transfer_from(Origin::signed(DAVE), SYMBOL.to_vec(), CHARLIE, BOB, amount),
			"no enough allowance"
		);

		assert_eq!(Htlc::get_balance_of(&id, &CHARLIE), amount);
		assert_eq!(Htlc::get_balance_of(&id, &BOB), 0);

		let approved_value = amount + 1;
		assert_ok!(Htlc::approve(
			Origin::signed(CHARLIE),
			SYMBOL.to_vec(),
			DAVE,
			approved_value
		));
		assert_eq!(Htlc::get_allowance(&id, &CHARLIE, &DAVE), approved_value);

		assert_err!(
			Htlc::transfer_from(
				Origin::signed(DAVE),
				SYMBOL.to_vec(),
				CHARLIE,
				BOB,
				approved_value
			),
			"no enough balance"
		);
		assert_eq!(Htlc::get_balance_of(&id, &CHARLIE), amount);
		assert_eq!(Htlc::get_balance_of(&id, &BOB), 0);
		assert_eq!(Htlc::get_allowance(&id, &CHARLIE, &DAVE), approved_value);

		assert_ok!(Htlc::transfer_from(
			Origin::signed(DAVE),
			SYMBOL.to_vec(),
			CHARLIE,
			BOB,
			amount
		));
		assert_eq!(Htlc::get_balance_of(&id, &CHARLIE), 0);
		assert_eq!(Htlc::get_balance_of(&id, &BOB), amount);
		assert_eq!(Htlc::get_allowance(&id, &CHARLIE, &DAVE), 1);

		assert_eq!(Htlc::total_supply(&id), amount);
	});
}
