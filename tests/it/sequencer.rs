use crate::helper::{
    cleanup_pod, init_environment,  wait_until_ready, init_stack,
};
use sequencer_relayer::sequencer::SequencerClient;

#[tokio::test]
async fn get_latest_block() {
    let podman = init_environment();
    let info = init_stack(&podman).await;
    wait_until_ready(&podman, &info.pod_name).await;
    let cosmos_endpoint = info.make_cosmos_endpoint();

    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    client.get_latest_block().await.unwrap();

    cleanup_pod(&podman, &info.pod_name).await;
}

#[tokio::test]
async fn get_block() {
    let podman = init_environment();
    let info = init_stack(&podman).await;
    wait_until_ready(&podman, &info.pod_name).await;
    let cosmos_endpoint = info.make_cosmos_endpoint();

    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    let resp = client.get_latest_block().await.unwrap();
    let height: u64 = resp.block.header.height.parse().unwrap();
    client.get_block(height).await.unwrap();

    cleanup_pod(&podman, &info.pod_name).await;
}
