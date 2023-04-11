use crate::helper::{
    cleanup_pod, init_runtime, start_pod_cli, wait_until_ready, PodDefinition, Ports,
};

use sequencer_relayer::{sequencer::SequencerClient, sequencer_block::SequencerBlock};

#[tokio::test]
async fn test_header_verify_hashes() {
    let podman = init_runtime();
    let pod_def = PodDefinition::get_randomized();
    let Ports { sequencer, .. } = start_pod_cli(&pod_def).await;
    let id = pod_def.name();
    wait_until_ready(&podman, id).await;
    let cosmos_endpoint = format!("http://127.0.0.1:{sequencer}");
    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    let resp = client.get_latest_block().await.unwrap();
    let sequencer_block = SequencerBlock::from_cosmos_block(resp.block).unwrap();
    sequencer_block.verify_data_hash().unwrap();
    sequencer_block.verify_block_hash().unwrap();
    cleanup_pod(&podman, &id).await;
}
