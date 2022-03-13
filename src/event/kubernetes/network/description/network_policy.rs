use crossbeam::channel::Sender;
use k8s_openapi::api::networking::v1::NetworkPolicy;

use crate::{
    error::Result,
    event::{
        kubernetes::{client::KubeClientRequest, network::NetworkMessage},
        Event,
    },
};

use super::DescriptionWorker;

pub(super) struct NetworkPolicyDescriptionWorker<'a, C>
where
    C: KubeClientRequest + Clone,
{
    client: &'a C,
    tx: &'a Sender<Event>,
    namespace: String,
    name: String,
}

#[async_trait::async_trait]
impl<'a, C> DescriptionWorker<'a, C> for NetworkPolicyDescriptionWorker<'a, C>
where
    C: KubeClientRequest + Clone,
{
    fn new(client: &'a C, tx: &'a Sender<Event>, namespace: String, name: String) -> Self {
        Self {
            client,
            tx,
            namespace,
            name,
        }
    }

    async fn run(&self) -> Result<()> {
        let url = format!(
            "apis/networking.k8s.io/v1/namespaces/{}/networkpolicies/{}",
            self.namespace, self.name
        );

        let res = self.client.request_text(&url).await?;

        let mut value: NetworkPolicy = serde_json::from_str(&res)?;

        value.metadata.managed_fields = None;

        let value = serde_yaml::to_string(&value)?
            .lines()
            .skip(1)
            .map(ToString::to_string)
            .collect();

        self.tx.send(NetworkMessage::Response(Ok(value)).into())?;

        Ok(())
    }
}
