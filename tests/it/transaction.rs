use sequencer_relayer::{sequencer::SequencerClient, transaction::txs_to_data_hash};

use crate::helper::{init_runtime, start_pod_cli, wait_until_ready, PodDefinition, Ports};

#[tokio::test]
async fn test_txs_to_data_hash() {
    let podman = init_runtime();
    let pod_def = PodDefinition::get_randomized();
    let Ports { sequencer, .. } = start_pod_cli(&pod_def).await;
    let id = pod_def.name();
    wait_until_ready(&podman, id).await;
    let cosmos_endpoint = format!("http://127.0.0.1:{sequencer}");
    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    let resp = client.get_latest_block().await.unwrap();
    let data_hash = txs_to_data_hash(&resp.block.data.txs);
    assert_eq!(
        data_hash.as_bytes(),
        &resp.block.header.data_hash.unwrap().0
    );
}
