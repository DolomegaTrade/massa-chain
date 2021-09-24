// Copyright (c) 2021 MASSA LABS <info@massa.net>

use crate::tests::tools::{self, generate_ledger_file};
use crypto::hash::Hash;
use models::ledger::LedgerData;
use models::{Address, Amount, BlockId, Endorsement, EndorsementContent, SerializeCompact, Slot};
use serial_test::serial;
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::test]
#[serial]
async fn test_reward_split() {
    // setup logging
    // stderrlog::new()
    // .verbosity(2)
    // .timestamp(stderrlog::Timestamp::Millisecond)
    // .init()
    // .unwrap();

    let thread_count = 2;

    // A
    let mut priv_a = crypto::generate_random_private_key();
    let mut pubkey_a = crypto::derive_public_key(&priv_a);
    let mut address_a = Address::from_public_key(&pubkey_a).unwrap();
    while 0 != address_a.get_thread(thread_count) {
        priv_a = crypto::generate_random_private_key();
        pubkey_a = crypto::derive_public_key(&priv_a);
        address_a = Address::from_public_key(&pubkey_a).unwrap();
    }

    // B
    let mut priv_b = crypto::generate_random_private_key();
    let mut pubkey_b = crypto::derive_public_key(&priv_b);
    let mut address_b = Address::from_public_key(&pubkey_b).unwrap();
    while 0 != address_b.get_thread(thread_count) {
        priv_b = crypto::generate_random_private_key();
        pubkey_b = crypto::derive_public_key(&priv_b);
        address_b = Address::from_public_key(&pubkey_b).unwrap();
    }

    let mut ledger = HashMap::new();
    ledger.insert(address_a, LedgerData::new(Amount::from_str("10").unwrap()));
    ledger.insert(address_b, LedgerData::new(Amount::from_str("10").unwrap()));
    let ledger_file = generate_ledger_file(&ledger);
    let staking_keys = vec![priv_a, priv_b];
    let staking_file = tools::generate_staking_keys_file(&staking_keys);
    let roll_counts_file = tools::generate_default_roll_counts_file(staking_keys.clone());
    let mut cfg = tools::default_consensus_config(
        ledger_file.path(),
        roll_counts_file.path(),
        staking_file.path(),
    );
    cfg.t0 = 500.into();
    cfg.delta_f0 = 32;
    cfg.thread_count = thread_count;
    cfg.operation_validity_periods = 10;
    cfg.operation_batch_size = 500;
    cfg.periods_per_cycle = 3;
    cfg.max_operations_per_block = 5000;
    cfg.max_block_size = 2000;
    cfg.endorsement_count = 5;
    cfg.block_reward = Amount::from_str("1").unwrap();

    tools::consensus_without_pool_test(
        cfg.clone(),
        None,
        async move |mut protocol_controller, consensus_command_sender, consensus_event_receiver| {
            // Check initial balances.
            let addresses_state = consensus_command_sender
                .get_addresses_info(vec![address_a, address_b].into_iter().collect())
                .await
                .unwrap();

            let addresse_a_state = addresses_state.get(&address_a).unwrap();
            assert_eq!(
                addresse_a_state.candidate_ledger_data.balance,
                Amount::from_str("10").unwrap()
            );

            let addresse_b_state = addresses_state.get(&address_b).unwrap();
            assert_eq!(
                addresse_b_state.candidate_ledger_data.balance,
                Amount::from_str("10").unwrap()
            );

            let draws: HashMap<_, _> = consensus_command_sender
                .get_selection_draws(Slot::new(1, 0), Slot::new(3, 0))
                .await
                .unwrap()
                .into_iter()
                .collect();

            let slot_one_block_addr = draws.get(&Slot::new(1, 0)).unwrap().0;
            let slot_one_endorsements_addrs = draws.get(&Slot::new(1, 0)).unwrap().1.clone();

            let (slot_one_pub_key, slot_one_priv_key) = if slot_one_block_addr == address_a {
                (pubkey_a, priv_a)
            } else {
                (pubkey_b, priv_b)
            };

            // Create, and propagate, block 1.
            let parents: Vec<BlockId> = consensus_command_sender
                .get_block_graph_status()
                .await
                .unwrap()
                .best_parents
                .iter()
                .map(|(b, _p)| *b)
                .collect();

            let (b1_id, b1, _) =
                tools::create_block(&cfg, Slot::new(1, 0), parents, slot_one_priv_key);

            tools::propagate_block(&mut protocol_controller, b1, true, 500).await;

            let slot_two_block_addr = draws.get(&Slot::new(2, 0)).unwrap().0;

            let (slot_two_pub_key, slot_two_priv_key) = if slot_two_block_addr == address_a {
                (pubkey_a, priv_a)
            } else {
                (pubkey_b, priv_b)
            };

            // Create, and propagate, block 2.
            let parents: Vec<BlockId> = consensus_command_sender
                .get_block_graph_status()
                .await
                .unwrap()
                .best_parents
                .iter()
                .map(|(b, _p)| *b)
                .collect();
            assert!(parents.contains(&b1_id));

            let (_b2_id, mut b2, _) =
                tools::create_block(&cfg, Slot::new(2, 0), parents, slot_two_priv_key);

            // Endorsements in block 2.

            // Creator of second block endorses the first.
            let index = slot_one_endorsements_addrs
                .iter()
                .position(|&addr| addr == Address::from_public_key(&slot_two_pub_key).unwrap())
                .unwrap() as u32;
            let content = EndorsementContent {
                sender_public_key: slot_two_pub_key,
                slot: Slot::new(1, 0),
                index,
                endorsed_block: b1_id,
            };
            let hash = Hash::hash(&content.to_bytes_compact().unwrap());
            let signature = crypto::sign(&hash, &slot_two_priv_key).unwrap();
            let ed_1 = Endorsement {
                content: content.clone(),
                signature,
            };

            // Creator of first block endorses the first.
            let index = slot_one_endorsements_addrs
                .iter()
                .position(|&addr| addr == Address::from_public_key(&slot_one_pub_key).unwrap())
                .unwrap() as u32;
            let content = EndorsementContent {
                sender_public_key: slot_one_pub_key,
                slot: Slot::new(1, 0),
                index,
                endorsed_block: b1_id,
            };
            let hash = Hash::hash(&content.to_bytes_compact().unwrap());
            let signature = crypto::sign(&hash, &slot_one_priv_key).unwrap();
            let ed_2 = Endorsement {
                content: content.clone(),
                signature,
            };

            // Creator of second block endorses the first, again.
            let index = slot_one_endorsements_addrs
                .iter()
                .position(|&addr| addr == Address::from_public_key(&slot_two_pub_key).unwrap())
                .unwrap() as u32;
            let content = EndorsementContent {
                sender_public_key: slot_two_pub_key,
                slot: Slot::new(1, 0),
                index,
                endorsed_block: b1_id,
            };
            let hash = Hash::hash(&content.to_bytes_compact().unwrap());
            let signature = crypto::sign(&hash, &slot_two_priv_key).unwrap();
            let ed_3 = Endorsement {
                content: content.clone(),
                signature,
            };

            // Add endorsements to block.
            b2.header.content.endorsements = vec![ed_1, ed_2, ed_3];

            // Propagate block.
            tools::propagate_block(&mut protocol_controller, b2, true, 500).await;

            // Check balances after second block.
            let addresses_state = consensus_command_sender
                .get_addresses_info(vec![address_a, address_b].into_iter().collect())
                .await
                .unwrap();

            let third = cfg
                .block_reward
                .checked_div_u64((3 * (1 + cfg.endorsement_count)).into())
                .unwrap();

            let expected_a = Amount::from_str("10")
                .unwrap() // initial ledger
                .saturating_add(if pubkey_a == slot_one_pub_key {
                    // block 1 reward
                    cfg.block_reward
                        .checked_div_u64((1 + cfg.endorsement_count).into())
                        .unwrap()
                        .saturating_sub(third)
                        // endorsements reward
                        .saturating_add(
                            third // parent in ed 1
                                .saturating_add(third) // creator of ed 2
                                .saturating_add(third) // parent in ed 2
                                .saturating_add(third), // parent in ed 3
                        )
                } else {
                    Default::default()
                })
                .saturating_add(if pubkey_a == slot_two_pub_key {
                    // block 2 creation reward
                    cfg.block_reward
                        .checked_mul_u64(1 + 3)
                        .unwrap()
                        .checked_div_u64((1 + cfg.endorsement_count).into())
                        .unwrap()
                        .saturating_sub(third.checked_mul_u64(2 * 3).unwrap())
                        // endorsement rewards
                        .saturating_add(
                            third // creator of ed 1
                                .saturating_add(third), //creator of ed 3
                        )
                } else {
                    Default::default()
                });

            let expected_b = Amount::from_str("10")
                .unwrap() // initial ledger
                .saturating_add(if pubkey_b == slot_one_pub_key {
                    // block 1 reward
                    cfg.block_reward
                        .checked_div_u64((1 + cfg.endorsement_count).into())
                        .unwrap()
                        .saturating_sub(third)
                        // endorsements reward
                        .saturating_add(
                            third // parent in ed 1
                                .saturating_add(third) // creator of ed 2
                                .saturating_add(third) // parent in ed 2
                                .saturating_add(third), // parent in ed 3
                        )
                } else {
                    Default::default()
                })
                .saturating_add(if pubkey_b == slot_two_pub_key {
                    // block 2 creation reward
                    cfg.block_reward
                        .checked_mul_u64(1 + 3)
                        .unwrap()
                        .checked_div_u64((1 + cfg.endorsement_count).into())
                        .unwrap()
                        .saturating_sub(third.checked_mul_u64(2 * 3).unwrap())
                        // endorsement rewards
                        .saturating_add(
                            third // creator of ed 1
                                .saturating_add(third), //creator of ed 3
                        )
                } else {
                    Default::default()
                });

            let state_a = addresses_state.get(&address_a).unwrap();
            assert_eq!(state_a.candidate_ledger_data.balance, expected_a);

            let state_b = addresses_state.get(&address_b).unwrap();
            assert_eq!(state_b.candidate_ledger_data.balance, expected_b);

            (
                protocol_controller,
                consensus_command_sender,
                consensus_event_receiver,
            )
        },
    )
    .await;
}
