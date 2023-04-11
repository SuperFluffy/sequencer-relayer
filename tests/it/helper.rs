use std::{
    path::PathBuf,
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};

use askama::Template;
use podman_api::{Id, Podman};
use uuid::Uuid;

static HOST_PORT: AtomicU16 = AtomicU16::new(1024);

#[derive(Template)]
#[template(path = "sequencer_relayer_stack.yaml.jinja2")]
struct SequencerRelayerStack<'a> {
    pod_name: &'a str,
    celestia_home_volume: &'a str,
    metro_home_volume: &'a str,
    scripts_host_volume: &'a str,
    bridge_host_port: u16,
    sequencer_host_port: u16,
}

pub fn init_environment() -> Podman {
    let uid = users::get_effective_uid();
    let podman_dir = PathBuf::from(format!("/run/user/{uid}/podman"));
    if podman_dir.exists() {
        Podman::unix(podman_dir.join("podman.sock"))
    } else {
        panic!("podman socket not found at `{}`", podman_dir.display(),);
    }
}

pub struct StackInfo {
    pub pod_name: String,
    pub bridge_host_port: u16,
    pub sequencer_host_port: u16,
}

impl StackInfo {
    pub fn make_cosmos_endpoint(&self) -> String {
        format!(
            "http://127.0.0.1:{}",
            self.sequencer_host_port,
        )
    }
}

pub async fn init_stack(podman: &Podman) -> StackInfo {
    let id = Uuid::new_v4().simple();
    let pod_name = format!("sequencer_relayer_stack-{id}");
    let celestia_home_volume = format!("celestia-home-volume-{id}");
    let metro_home_volume = format!("metro-home-volume-{id}");
    let bridge_host_port = HOST_PORT.fetch_add(1, Ordering::Relaxed);
    let sequencer_host_port = HOST_PORT.fetch_add(1, Ordering::Relaxed);

    let scripts_host_volume = format!("{}/containers/", env!("CARGO_MANIFEST_DIR"));

    let stack = SequencerRelayerStack {
        pod_name: &pod_name,
        celestia_home_volume: &celestia_home_volume,
        metro_home_volume: &metro_home_volume,
        scripts_host_volume: &scripts_host_volume,
        bridge_host_port,
        sequencer_host_port,
    };

    let pod_kube_yaml = stack.render().unwrap();

    let _report = podman
        .play_kubernetes_yaml(&Default::default(), pod_kube_yaml)
        .await
        .unwrap();

    StackInfo {
        pod_name,
        bridge_host_port,
        sequencer_host_port,
    }
}


pub async fn cleanup_pod(podman: &Podman, name: &str) {
    let _ = podman.pods().get(name).remove().await;
}

pub async fn wait_until_ready(podman: &Podman, id: impl Into<Id>) {
    let pod = podman.pods().get(id);
    loop {
        let resp = pod.inspect().await.unwrap();
        if resp.state.as_deref() == Some("Running") {
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    // we need to sleep to ensure that we have blocks available
    // FIXME: this needs a more reliable mechanism
    tokio::time::sleep(Duration::from_secs(20)).await;
}
