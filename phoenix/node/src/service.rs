use cumulus_client_consensus_relay_chain::{
	build_relay_chain_consensus, BuildRelayChainConsensusParams,
};
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use polkadot_primitives::v0::CollatorPair;
use parachain_runtime::{RuntimeApi, opaque::Block};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager,BasePath};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use sp_keystore::SyncCryptoStore;
use fc_consensus::FrontierBlockImport;
use sc_cli::SubstrateCli;
use fc_rpc_core::types::{FilterPool, PendingTransactions};

use std::{
        collections::{HashMap,BTreeMap},
        sync::{Arc, Mutex},
};

// Native executor instance.
native_executor_instance!(
	pub Executor,
	parachain_runtime::api::dispatch,
	parachain_runtime::native_version,
);

pub fn open_frontier_backend(config: &Configuration) -> Result<Arc<fc_db::Backend<Block>>, String> {
	let config_dir = config.base_path.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.unwrap_or_else(|| {
			BasePath::from_project("", "", &crate::cli::Cli::executable_name())
				.config_dir(config.chain_spec.id())
		});
	let database_dir = config_dir.join("frontier").join("db");

	Ok(Arc::new(fc_db::Backend::<Block>::new(&fc_db::DatabaseSettings {
		source: fc_db::DatabaseSettingsSrc::RocksDb {
			path: database_dir,
			cache_size: 0,
		}
	})?))
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial(
	config: &Configuration,
) -> Result<
	PartialComponents<
		TFullClient<Block, RuntimeApi, Executor>,
		TFullBackend<Block>,
		(),
		sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		sc_transaction_pool::FullPool<Block, TFullClient<Block, RuntimeApi, Executor>>,
		(
                    Option<Telemetry>, Option<TelemetryWorkerHandle>,
                    PendingTransactions,
                    Option<FilterPool>, 
                    Arc<fc_db::Backend<Block>>
                ),
	>,
	sc_service::Error,
> {
	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let telemetry = config
                .telemetry_endpoints
                .clone()
                .filter(|x| !x.is_empty())
                .map(|endpoints| -> Result<_, sc_telemetry::Error> {
                        let worker = TelemetryWorker::new(16)?;
                        let telemetry = worker.handle().new_telemetry(endpoints);
                        Ok((worker, telemetry))
                })
                .transpose()?;

        let (client, backend, keystore_container, task_manager) = 
             sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
                &config,
                telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;

	let client = Arc::new(client);
	let registry = config.prometheus_registry();


        let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

        let telemetry = telemetry.map(|(worker, telemetry)| {
                task_manager.spawn_handle().spawn("telemetry", worker.run());
                telemetry
        });


        // auto register offchain key, add by WangYi
        let keystore = keystore_container.sync_keystore();
        SyncCryptoStore::sr25519_generate_new(&*keystore, gateway::KEY_TYPE, Some("//Alice"))
                       .expect("Creating key with account Alice should succeed.");
        SyncCryptoStore::sr25519_generate_new(&*keystore, price_fetch::KEY_TYPE, Some("//Alice"))
                       .expect("Creating key with account Alice should succeed.");


	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

        // add by WangYi
        let frontier_backend = open_frontier_backend(config)?;
        let pending_transactions: PendingTransactions = Some(Arc::new(Mutex::new(HashMap::new())));
	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
        let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone(),frontier_backend.clone() );

	let import_queue = cumulus_client_consensus_relay_chain::import_queue(
		client.clone(),
		frontier_block_import.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_essential_handle(),
		registry.clone(),
	)?;

	let params = PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		inherent_data_providers,
		select_chain: (),
		other: (telemetry,telemetry_worker_handle,pending_transactions,filter_pool,frontier_backend),
	};

	Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RB>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: ParaId,
	collator: bool,
	_rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RB: Fn(
			Arc<TFullClient<Block, RuntimeApi, Executor>>,
		) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
		+ Send
		+ 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config)?;
	params
		.inherent_data_providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.unwrap();

        let (mut telemetry,telemetry_worker_handle,pending_transactions,filter_pool,frontier_backend) = params.other;

        let polkadot_full_node = cumulus_client_service::build_polkadot_full_node(
                polkadot_config,
                collator_key.clone(),
                telemetry_worker_handle,
        )
        .map_err(|e| match e {
                polkadot_service::Error::Sub(x) => x,
                s => format!("{}", s).into(),
        })?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		polkadot_full_node.client.clone(),
		id,
		Box::new(polkadot_full_node.network.clone()),
		polkadot_full_node.backend.clone(),
	);

	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = params.import_queue;
	let (network, network_status_sinks, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;
        /*
	let rpc_client = client.clone();
	let rpc_extensions_builder = Box::new(move |_, _| rpc_ext_builder(rpc_client.clone()));
        */
        // Add by WangYi
        let subscription_task_executor = sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
        let rpc_extensions_builder = {
                let client = client.clone();
                let pool = transaction_pool.clone();
                let network = network.clone();
                let pending = pending_transactions.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();

                Box::new(move |deny_unsafe, _| {
                        let deps = crate::rpc::FullDeps {
                                client: client.clone(),
                                pool: pool.clone(),
                                deny_unsafe,
                                is_authority: collator,
                                network: network.clone(),
                                pending_transactions: pending.clone(),
                                filter_pool: filter_pool.clone(),
				backend: frontier_backend.clone(),
                        };

                        crate::rpc::create_full(deps, subscription_task_executor.clone())
                })
        };


        // enable offchain workers hook add by WangYi
        sc_service::build_offchain_workers(
                &parachain_config, task_manager.spawn_handle(), client.clone(), network.clone(),
        );

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: None,
		remote_blockchain: None,
		rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		network_status_sinks,
		system_rpc_tx,
                telemetry: telemetry.as_mut(),
	})?;

        let announce_block = {
                let network = network.clone();
                Arc::new(move |hash, data| network.announce_block(hash, data))
        };

	if collator {
		let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
                        telemetry.as_ref().map(|x| x.handle()),
		);
		let spawner = task_manager.spawn_handle();

		let parachain_consensus = build_relay_chain_consensus(BuildRelayChainConsensusParams {
			para_id: id,
			proposer_factory,
			inherent_data_providers: params.inherent_data_providers,
			block_import: client.clone(),
			relay_chain_client: polkadot_full_node.client.clone(),
			relay_chain_backend: polkadot_full_node.backend.clone(),
		});

		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			collator_key,
			relay_chain_full_node: polkadot_full_node,
			spawner,
			backend,
			parachain_consensus,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			polkadot_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a normal parachain node.
pub async fn start_node(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: ParaId,
	validator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)> {
	start_node_impl(
		parachain_config,
		collator_key,
		polkadot_config,
		id,
		validator,
		|_| Default::default(),
	)
	.await
}