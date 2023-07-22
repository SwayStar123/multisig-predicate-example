use fuels::{accounts::predicate::Predicate, prelude::*, types::Bits256};

abigen!(
    Predicate(
        name = "MyPredicate",
        abi = "./out/debug/multisig-predicate-example-abi.json",
    ),
);

async fn setup() -> (WalletUnlocked, WalletUnlocked, WalletUnlocked, Predicate) {
    let wallets_config = WalletsConfig::new_multiple_assets(
        3,
        vec![
            AssetConfig {
                id: AssetId::default(),
                num_coins: DEFAULT_NUM_COINS,
                coin_amount: DEFAULT_COIN_AMOUNT,
            }
        ],
    );

    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, None, None).await;
    let (a, b, c) = (wallets[0].clone(), wallets[1].clone(), wallets[2].clone());

    let predicate_data = MyPredicateEncoder::encode_data();
    let code_path = "./out/debug/multisig-predicate-example-abi.bin";

    let predicate_configurables = MyPredicateConfigurables::new()
        .set_ADDRESS_ONE(Bits256(*Address::from(a.address())))
        .set_ADDRESS_TWO(Bits256(*Address::from(b.address())))
        .set_ADDRESS_THREE(Bits256(*Address::from(c.address())));

    let predicate: Predicate = Predicate::load_from(code_path)
        .unwrap()
        .with_data(predicate_data)
        .with_provider(a.try_provider().unwrap().clone())
        .with_configurables(predicate_configurables);

    (a, b, c, predicate)
}

