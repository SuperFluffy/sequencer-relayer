use crate::helper::{
    cleanup_pod, init_runtime, start_pod_cli, wait_until_ready, PodDefinition, Ports,
};
use sequencer_relayer::sequencer::SequencerClient;

#[tokio::test]
async fn get_latest_block() {
    let podman = init_runtime();
    let pod_def = PodDefinition::get_randomized();
    let Ports { sequencer, .. } = start_pod_cli(&pod_def).await;
    let id = pod_def.name();
    wait_until_ready(&podman, id).await;
    let cosmos_endpoint = format!("http://127.0.0.1:{sequencer}");
    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    client.get_latest_block().await.unwrap();
    cleanup_pod(&podman, &id).await;
}

#[tokio::test]
async fn get_block() {
    let podman = init_runtime();
    let pod_def = PodDefinition::get_randomized();
    let Ports { sequencer, .. } = start_pod_cli(&pod_def).await;
    let id = pod_def.name();
    wait_until_ready(&podman, id).await;
    let cosmos_endpoint = format!("http://127.0.0.1:{sequencer}");
    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    let resp = client.get_latest_block().await.unwrap();
    let height: u64 = resp.block.header.height.parse().unwrap();
    client.get_block(height).await.unwrap();
    cleanup_pod(&podman, &id).await;
}
