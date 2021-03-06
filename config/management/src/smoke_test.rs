// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, layout::Layout, storage_helper::StorageHelper};
use config_builder::{BuildSwarm, SwarmConfig};
use libra_config::{
    config::{
        DiscoveryMethod, Identity, NetworkConfig, NodeConfig, OnDiskStorageConfig, RoleType,
        SecureBackend, SeedPeersConfig, WaypointConfig, HANDSHAKE_VERSION,
    },
    network_id::NetworkId,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_swarm::swarm::{LibraNode, LibraSwarm, LibraSwarmDir};
use libra_temppath::TempPath;
use libra_types::account_address;
use std::path::{Path, PathBuf};

struct ManagementBuilder {
    configs: Vec<NodeConfig>,
    faucet_key: Ed25519PrivateKey,
}

impl BuildSwarm for ManagementBuilder {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        Ok((self.configs.clone(), self.faucet_key.clone()))
    }
}

#[test]
fn smoke_test() {
    LibraNode::prepare();
    let helper = StorageHelper::new();
    let num_validators = 5;
    let shared = "_shared";
    let association = "association";
    let association_shared = association.to_string() + shared;

    // Step 1) Prepare the layout
    let mut layout = Layout::default();
    layout.association = vec![association_shared.to_string()];
    layout.operators = (0..num_validators)
        .map(|v| (v.to_string() + shared))
        .collect();

    let mut common_storage = helper.storage(crate::constants::COMMON_NS.into());
    let layout_value = Value::String(layout.to_toml().unwrap());
    common_storage
        .set(crate::constants::LAYOUT, layout_value)
        .unwrap();

    // Step 2) Set association key
    helper.initialize(association.into());
    helper
        .association_key(&association, &association_shared)
        .unwrap();

    // Step 3) Prepare validators
    let temppath = TempPath::new();
    temppath.create_as_dir().unwrap();
    let swarm_path = temppath.path().to_path_buf();

    let mut configs = Vec::new();
    for i in 0..num_validators {
        let ns = i.to_string();
        let ns_shared = ns.clone() + shared;
        helper.initialize(ns.clone());

        let operator_key = helper.operator_key(&ns, &ns_shared).unwrap();

        let validator_account = account_address::from_public_key(&operator_key);
        let mut config = NodeConfig::default();

        let mut network = NetworkConfig::network_with_id(NetworkId::Validator);
        network.discovery_method = DiscoveryMethod::None;
        network.mutual_authentication = true;
        config.validator_network = Some(network);

        let mut network = NetworkConfig::network_with_id(NetworkId::Public);
        network.discovery_method = DiscoveryMethod::None;
        config.full_node_networks = vec![network];
        config.randomize_ports();

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        validator_network.identity = Identity::from_storage(
            libra_global_constants::VALIDATOR_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            secure_backend(helper.path(), &swarm_path, &ns, "validator"),
        );

        let fullnode_network = &mut config.full_node_networks[0];
        let fullnode_network_address = fullnode_network.listen_address.clone();
        fullnode_network.identity = Identity::from_storage(
            libra_global_constants::FULLNODE_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            secure_backend(helper.path(), &swarm_path, &ns, "full_node"),
        );

        configs.push(config);

        helper.operator_key(&ns, &ns_shared).unwrap();
        helper
            .validator_config(
                validator_account,
                validator_network_address,
                fullnode_network_address,
                &ns,
                &ns_shared,
            )
            .unwrap();
    }

    // Step 4) Produce genesis and introduce into node configs
    let genesis_path = TempPath::new();
    genesis_path.create_as_file().unwrap();
    let genesis = helper.genesis(genesis_path.path()).unwrap();

    // Save the waypoint into shared secure storage so that validators can perform insert_waypoint
    let waypoint = helper.create_waypoint(constants::COMMON_NS).unwrap();

    // Step 5) Introduce waypoint and genesis into the configs and verify along the way
    for (i, mut config) in configs.iter_mut().enumerate() {
        let ns = i.to_string();
        helper.insert_waypoint(&ns, constants::COMMON_NS).unwrap();
        let output = helper.verify_genesis(&ns, genesis_path.path()).unwrap();
        // 4 matches = 5 splits
        assert_eq!(output.split("match").count(), 5);

        config.consensus.safety_rules.backend =
            secure_backend(helper.path(), &swarm_path, &ns, "safety-rules");

        if i == 0 {
            // This is unfortunate due to the way SwarmConfig works
            config.base.waypoint = WaypointConfig::FromConfig(waypoint);
        } else {
            let backend = secure_backend(helper.path(), &swarm_path, &ns, "waypoint");
            config.base.waypoint = WaypointConfig::FromStorage(backend);
        }
        config.execution.genesis = Some(genesis.clone());
    }

    // Step 6) Prepare ecosystem
    let full_node_config =
        attach_validator_full_node(&helper, &mut configs[0], &swarm_path, 0.to_string());

    // Step 7) Build configuration for Swarm
    let faucet_key = helper
        .storage(association.into())
        .export_private_key(libra_global_constants::ASSOCIATION_KEY)
        .unwrap();
    let management_builder = ManagementBuilder {
        configs,
        faucet_key: faucet_key.clone(),
    };

    let mut swarm = LibraSwarm {
        dir: LibraSwarmDir::Temporary(temppath),
        nodes: std::collections::HashMap::new(),
        config: SwarmConfig::build(&management_builder, &swarm_path).unwrap(),
    };

    // Step 8) Launch validators
    swarm.launch_attempt(RoleType::Validator, false).unwrap();

    // Step 9) Launch ecosystem
    let management_builder = ManagementBuilder {
        configs: vec![full_node_config],
        faucet_key,
    };

    let temppath = TempPath::new();
    temppath.create_as_dir().unwrap();
    let swarm_path = temppath.path().to_path_buf();

    let mut swarm = LibraSwarm {
        dir: LibraSwarmDir::Temporary(temppath),
        nodes: std::collections::HashMap::new(),
        config: SwarmConfig::build(&management_builder, &swarm_path).unwrap(),
    };

    swarm.launch_attempt(RoleType::FullNode, false).unwrap();
    assert!(check_connectivity(&mut swarm, 1));
}

fn check_connectivity(swarm: &mut LibraSwarm, expected_peers: i64) -> bool {
    for node in swarm.nodes.iter_mut() {
        let mut timed_out = true;
        for i in 0..60 {
            println!("Wait for connectivity attempt: {} {}", node.0, i);
            std::thread::sleep(std::time::Duration::from_secs(1));
            if node.1.check_connectivity(expected_peers) {
                timed_out = false;
                break;
            }
        }
        if timed_out {
            return false;
        }
    }
    true
}

fn attach_validator_full_node(
    helper: &StorageHelper,
    validator_config: &mut NodeConfig,
    swarm_path: &Path,
    ns: String,
) -> NodeConfig {
    // Create two vfns, we'll pass one to the validator later
    let fn_vfn = NetworkConfig::network_with_id(NetworkId::vfn_network());
    let v_vfn = NetworkConfig::network_with_id(NetworkId::vfn_network());

    let mut full_node_config = NodeConfig::default();
    full_node_config.full_node_networks = vec![fn_vfn, v_vfn];
    full_node_config.randomize_ports();

    // Now let's swap the full node network on the validator with the PFN on the fullnode, since
    // that's the VFN and what we want clients to connect to.
    let internal_fn = full_node_config.full_node_networks.swap_remove(1);
    let pfn = validator_config.full_node_networks.swap_remove(0);
    full_node_config.full_node_networks.push(pfn);
    validator_config.full_node_networks.push(internal_fn);

    // Now let's prepare the full nodes internal network to communicate with the validators
    // internal network

    let v_vfn = &mut validator_config.full_node_networks[0];
    v_vfn.discovery_method = DiscoveryMethod::None;
    v_vfn.identity = Identity::from_storage(
        libra_global_constants::FULLNODE_NETWORK_KEY.into(),
        libra_global_constants::OPERATOR_ACCOUNT.into(),
        secure_backend(helper.path(), &swarm_path, &ns, "pfn_full_node"),
    );

    let v_vfn_network_address = v_vfn.listen_address.clone();
    let v_vfn_pub_key = libra_secure_storage::config::identity_key(v_vfn).public_key();
    let v_vfn_network_address =
        v_vfn_network_address.append_prod_protos(v_vfn_pub_key, HANDSHAKE_VERSION);
    let v_vfn_id = libra_secure_storage::config::peer_id(v_vfn);
    let mut seed_peers = SeedPeersConfig::default();
    seed_peers.insert(v_vfn_id, vec![v_vfn_network_address]);

    let fn_vfn = &mut full_node_config.full_node_networks[0];
    fn_vfn.discovery_method = DiscoveryMethod::None;
    fn_vfn.seed_peers = seed_peers;

    full_node_config.base.waypoint = validator_config.base.waypoint.clone();
    full_node_config.base.role = RoleType::FullNode;
    full_node_config.execution.genesis = validator_config.execution.genesis.clone();
    full_node_config
}

fn secure_backend(original: &Path, dst_base: &Path, ns: &str, usage: &str) -> SecureBackend {
    let mut dst = dst_base.to_path_buf();
    dst.push(format!("{}_{}", usage, ns));
    std::fs::copy(original, &dst).unwrap();

    let mut storage_config = OnDiskStorageConfig::default();
    storage_config.path = dst;
    storage_config.set_data_dir(PathBuf::from(""));
    storage_config.namespace = Some(ns.into());
    SecureBackend::OnDiskStorage(storage_config)
}
