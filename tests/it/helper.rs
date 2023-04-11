use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};

use podman_api::{Id, Podman};
use serde::Deserialize as _;
use serde_yaml::Value;
use tokio::io::AsyncWriteExt as _;
use uuid::Uuid;

static HOST_PORT: AtomicU16 = AtomicU16::new(1024);

pub fn init_runtime() -> Podman {
    let uid = users::get_effective_uid();
    let podman_dir = PathBuf::from(format!("/run/user/{uid}/podman"));
    if podman_dir.exists() {
        Podman::unix(podman_dir.join("podman.sock"))
    } else {
        panic!("podman socket not found at `{}`", podman_dir.display(),);
    }
}

pub struct Ports {
    pub sequencer: u16,
    pub bridge: u16,
}
pub async fn start_pod_cli(pod_def: &PodDefinition) -> Ports {
    let yaml = pod_def.to_string();
    let sequencer = HOST_PORT.fetch_add(1, Ordering::Relaxed);
    let bridge = HOST_PORT.fetch_add(1, Ordering::Relaxed);
    let mut cmd = tokio::process::Command::new("podman")
        .arg("play")
        .arg("kube")
        .arg(format!("--publish={sequencer}:1318"))
        .arg(format!("--publish={bridge}:26659"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(yaml.as_bytes())
        .await
        .unwrap();
    cmd.stdin.as_mut().unwrap().flush().await.unwrap();
    cmd.wait_with_output().await.unwrap();
    Ports { bridge, sequencer }
}

#[derive(Clone, Debug)]
pub struct PodDefinition {
    pub celestia_cfg_map: Value,
    pub metro_cfg_map: Value,
    pub pod_def: Value,
}

impl PodDefinition {
    pub fn name(&self) -> &str {
        self.pod_def
            .get("metadata")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
    }

    /// get the pod definition from docker/test_sequencer_relayer.yaml
    pub fn get() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let pod_definition_path =
            Path::new(manifest_dir).join("docker/test_sequencer_relayer.yaml");
        let f = File::open(pod_definition_path).unwrap();
        let mut deser = serde_yaml::Deserializer::from_reader(f);
        let celestia_cfg_map = Value::deserialize(deser.next().unwrap()).unwrap();
        let metro_cfg_map = Value::deserialize(deser.next().unwrap()).unwrap();
        let pod_def = Value::deserialize(deser.next().unwrap()).unwrap();
        Self {
            celestia_cfg_map,
            metro_cfg_map,
            pod_def,
        }
    }

    /// like `PodDefinition::get`, but randomize the pod name and emptyDir volume mounts
    ///
    /// this avoids name clashes when podman tries to spin up several pods with the same
    /// name and/or empty dir volumes
    pub fn get_randomized() -> Self {
        let mut this = Self::get();

        // rename the pod name
        let pod_name = this
            .pod_def
            .get_mut("metadata")
            .unwrap()
            .get_mut("name")
            .unwrap();
        let id = Uuid::new_v4().simple();
        *pod_name = Value::String(format!(
            "{pod_name}-{id}",
            pod_name = pod_name.as_str().unwrap()
        ));

        // rename empty dir volumes so podman does not clash when creating multiple dirs for
        // different pod instances
        let empty_dir_volume_names = this
            .pod_def
            .get_mut("spec")
            .unwrap()
            .get_mut("volumes")
            .unwrap()
            .as_sequence_mut()
            .unwrap()
            .iter_mut()
            .filter(|vol| vol.get("emptyDir").is_some())
            .map(|vol| {
                let vol_name = vol.get_mut("name").unwrap();
                let orig_name = vol_name.as_str().unwrap().to_string();
                let new_name = format!("{vol_name}-{id}", vol_name = vol_name.as_str().unwrap());
                *vol_name = Value::String(new_name.clone());
                (orig_name, new_name)
            })
            .collect::<HashMap<_, _>>();

        // set the volume mounts to the new names
        // first for the init containers
        this.pod_def
            .get_mut("spec")
            .unwrap()
            .get_mut("initContainers")
            .unwrap()
            .as_sequence_mut()
            .unwrap()
            .iter_mut()
            .for_each(|container| {
                container.get_mut("volumeMounts").map(|mounts| {
                    mounts
                        .as_sequence_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|mount| {
                            let name = mount.get_mut("name").unwrap();
                            empty_dir_volume_names
                                .get(name.as_str().unwrap())
                                .cloned()
                                .map(|new_name| *name = Value::String(new_name));
                        })
                });
            });
        // second for the normal containers
        this.pod_def
            .get_mut("spec")
            .unwrap()
            .get_mut("containers")
            .unwrap()
            .as_sequence_mut()
            .unwrap()
            .iter_mut()
            .for_each(|container| {
                container.get_mut("volumeMounts").map(|mounts| {
                    mounts
                        .as_sequence_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|mount| {
                            let name = mount.get_mut("name").unwrap();
                            empty_dir_volume_names
                                .get(name.as_str().unwrap())
                                .cloned()
                                .map(|new_name| *name = Value::String(new_name));
                        })
                });
            });
        this
    }

    pub fn to_string(&self) -> String {
        let mut yaml_buf = vec![];
        serde_yaml::to_writer(&mut yaml_buf, &self.celestia_cfg_map).unwrap();
        yaml_buf.extend_from_slice(b"\n---\n");
        serde_yaml::to_writer(&mut yaml_buf, &self.metro_cfg_map).unwrap();
        yaml_buf.extend_from_slice(b"\n---\n");
        serde_yaml::to_writer(&mut yaml_buf, &self.pod_def).unwrap();
        String::from_utf8(yaml_buf).unwrap()
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
