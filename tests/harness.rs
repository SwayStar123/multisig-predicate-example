use fuels::{
    accounts::predicate::Predicate,
    prelude::*,
    tx::{Bytes32, Receipt},
    types::{
        transaction_builders::{ScriptTransactionBuilder, TransactionBuilder},
        Bits256,
    },
};

abigen!(Predicate(
    name = "MyPredicate",
    abi = "./out/debug/multisig-predicate-example-abi.json",
),);

async fn setup() -> (WalletUnlocked, WalletUnlocked, WalletUnlocked, Predicate) {
    let wallets_config = WalletsConfig::new_multiple_assets(
        4,
        vec![AssetConfig {
            id: AssetId::default(),
            num_coins: DEFAULT_NUM_COINS,
            coin_amount: DEFAULT_COIN_AMOUNT,
        }],
    );

    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, None, None).await;
    let (a, b, c, d) = (
        wallets[0].clone(),
        wallets[1].clone(),
        wallets[2].clone(),
        wallets[3].clone(),
    );

    let predicate_data = MyPredicateEncoder::encode_data();
    let code_path = "./out/debug/multisig-predicate-example.bin";

    let predicate_configurables = MyPredicateConfigurables::new()
        .set_ADDRESS_ONE(Bits256(*Address::from(a.address())))
        .set_ADDRESS_TWO(Bits256(*Address::from(b.address())))
        .set_ADDRESS_THREE(Bits256(*Address::from(c.address())));

    let predicate: Predicate = Predicate::load_from(code_path)
        .unwrap()
        .with_data(predicate_data)
        .with_provider(a.try_provider().unwrap().clone())
        .with_configurables(predicate_configurables);

    d.transfer(
        predicate.address(),
        DEFAULT_COIN_AMOUNT,
        AssetId::default(),
        TxParameters::default(),
    )
    .await
    .unwrap();

    (a, b, c, predicate)
}

async fn make_predicate_spend_tx(
    predicate: &Predicate,
    to: &Bech32Address,
    amount: u64,
    asset_id: AssetId,
    tx_parameters: TxParameters,
) -> ScriptTransaction {
    let inputs = predicate
        .get_asset_inputs_for_amount(asset_id, amount, None)
        .await
        .unwrap();

    let outputs = predicate.get_asset_outputs_for_amount(to, asset_id, amount);

    let consensus_parameters = predicate.try_provider().unwrap().consensus_parameters();

    let tx_builder = ScriptTransactionBuilder::prepare_transfer(inputs, outputs, tx_parameters)
        .set_consensus_parameters(consensus_parameters);

    // if we are not transferring the base asset, previous base amount is 0
    let previous_base_amount = if asset_id == AssetId::default() {
        amount
    } else {
        0
    };

    let tx = predicate
        .add_fee_resources(tx_builder, previous_base_amount, None)
        .await
        .unwrap();

    tx
}

async fn send_tx(provider: &Provider, tx: &ScriptTransaction) -> (Bytes32, Vec<Receipt>) {
    let receipts = provider.send_transaction(tx).await.unwrap();
    let consensus_parameters = provider.consensus_parameters();

    (tx.id(consensus_parameters.chain_id.into()), receipts)
}

#[tokio::test]
async fn dual_signer() {
    let (a, b, _c, predicate) = setup().await;

    let mut tx = make_predicate_spend_tx(
        &predicate,
        &a.address(),
        DEFAULT_COIN_AMOUNT,
        AssetId::default(),
        TxParameters::default(),
    )
    .await;

    a.sign_transaction(&mut tx).unwrap();
    b.sign_transaction(&mut tx).unwrap();

    send_tx(predicate.provider().unwrap(), &tx).await;

    let balance = a.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(balance, DEFAULT_COIN_AMOUNT + DEFAULT_COIN_AMOUNT);
}

