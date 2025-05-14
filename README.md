// let mut evm = Evm::builder()
        //     .with_db(db)
        //     .modify_tx_env(|tx| {
        //         tx.caller = address!("000000000000000000000000000000000000000A");
        //         tx.transact_to = TransactTo::Create;
        //         tx.data = init_code;
        //         tx.value = U256::from(0);
        //     })
        //     .modify_cfg_env(|cfg| cfg.limit_contract_code_size = Some(usize::MAX))
        //     .append_handler_register(handle_register)
        //     .build();