use std::time::Duration;

use massa_models::{
    address::Address,
    amount::Amount,
    config::THREAD_COUNT,
    operation::{Operation, OperationType},
};
use massa_pool_exports::PoolController;
use massa_protocol_exports::ProtocolController;
use massa_signature::KeyPair;
use massa_storage::Storage;
use massa_time::MassaTime;
use massa_wallet::Wallet;

pub fn start_operation_injector(
    genesis_timestamp: MassaTime,
    storage: Storage,
    mut wallet: Wallet,
    mut pool_controller: Box<dyn PoolController>,
    protocol_controller: Box<dyn ProtocolController>,
) {
    std::thread::sleep(
        genesis_timestamp
            .clone()
            .saturating_sub(MassaTime::now().unwrap())
            .saturating_add(MassaTime::from_millis(1000))
            .to_duration(),
    );
    let return_addr = wallet
        .get_wallet_address_list()
        .iter()
        .next()
        .unwrap()
        .clone();
    let mut distant_wallets = vec![KeyPair::generate(0).unwrap(); 32];
    let mut wallets_created = vec![false; 32];
    let mut init_ops = vec![];
    while wallets_created.iter().any(|e| *e == false) {
        let keypair = KeyPair::generate(0).unwrap();
        let finals = pool_controller.get_final_cs_periods();
        let addr = Address::from_public_key(&keypair.get_public_key());
        let index: usize = addr.get_thread(THREAD_COUNT) as usize;
        if !wallets_created[index] {
            distant_wallets[index] = keypair;
            wallets_created[index] = true;
            init_ops.push(
                wallet
                    .create_operation(
                        Operation {
                            fee: Amount::from_mantissa_scale(0, 0),
                            expire_period: finals[index] + 10,
                            op: OperationType::Transaction {
                                recipient_address: addr,
                                amount: Amount::from_mantissa_scale(10000, 0),
                            },
                        },
                        return_addr,
                    )
                    .unwrap(),
            )
        }
    }
    println!("Sending init ops len: {}", init_ops.len());
    let mut storage = storage.clone_without_refs();
    storage.store_operations(init_ops);
    pool_controller.add_operations(storage.clone());
    protocol_controller
        .propagate_operations(storage.clone())
        .unwrap();
    wallet.add_keypairs(distant_wallets.clone()).unwrap();
    std::thread::spawn(move || {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        loop {
            let mut storage = storage.clone_without_refs();
            let txps = 1500 / 32;
            let finals = pool_controller.get_final_cs_periods();
            let mut ops = vec![];

            for i in 0..32 {
                for _ in 0..txps {
                    let amount = rng.gen_range(1..=10000);
                    let content = Operation {
                        fee: Amount::from_mantissa_scale(0, 0),
                        expire_period: finals[i] + 10,
                        op: OperationType::Transaction {
                            recipient_address: return_addr,
                            amount: Amount::from_mantissa_scale(amount, 8),
                        },
                    };
                    let address = Address::from_public_key(&distant_wallets[i].get_public_key());
                    ops.push(wallet.create_operation(content, address).unwrap())
                }
            }
            println!("Sending ops len: {}", ops.len());
            storage.store_operations(ops);
            pool_controller.add_operations(storage.clone());
            protocol_controller
                .propagate_operations(storage.clone())
                .unwrap();
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}
