use crate::helper::{init_runtime, start_pod_cli, wait_until_ready, PodDefinition, Ports};
use sequencer_relayer::sequencer::SequencerClient;

#[tokio::test]
async fn test_header_to_tendermint_header() {
    let podman = init_runtime();
    let pod_def = PodDefinition::get_randomized();
    let Ports { sequencer, .. } = start_pod_cli(&pod_def).await;
    let id = pod_def.name();
    wait_until_ready(&podman, id).await;
    let cosmos_endpoint = format!("http://127.0.0.1:{sequencer}");
    let client = SequencerClient::new(cosmos_endpoint).unwrap();
    let resp = client.get_latest_block().await.unwrap();
    let tm_header = &resp.block.header.to_tendermint_header().unwrap();
    let tm_header_hash = tm_header.hash();
    assert_eq!(tm_header_hash.as_bytes(), &resp.block_id.hash.0);
}
