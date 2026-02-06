use std::sync::Arc;

use genai::{
    ModelIden, ModelName, ServiceTarget,
    adapter::AdapterKind,
    chat::{ChatMessage, ChatRequest, MessageContent},
    resolver::{AuthData, AuthResolver, AuthResolverFn, Endpoint, ServiceTargetResolver},
};

pub mod caption;
pub mod translation;

pub trait FromUserConfiguredKey {
    fn from_user_configured_key(key: String) -> Self;
}

#[derive(Debug, Clone)]
struct ConstantAuthResolver(String);

impl AuthResolverFn for ConstantAuthResolver {
    fn exec_fn(&self, _model_iden: ModelIden) -> genai::resolver::Result<Option<AuthData>> {
        Ok(Some(AuthData::Key(self.0.clone())))
    }

    fn clone_box(&self) -> Box<dyn AuthResolverFn> {
        Box::new(self.clone())
    }
}

impl FromUserConfiguredKey for genai::Client {
    fn from_user_configured_key(key: String) -> Self {
        let target_resolver = ServiceTargetResolver::from_resolver_fn(|target| {
            let ServiceTarget {
                mut model,
                auth,
                endpoint,
            } = target;
            let (namespace, model_name) = model.model_name.namespace_and_name();
            if let Some((adapter, id)) = model_name.split_once('/') {
                model.adapter_kind = AdapterKind::from_lower_str(adapter).unwrap();
                model.model_name = ModelName::from(if let Some(namespace) = namespace {
                    format!("{}::{}", namespace, id)
                } else {
                    id.to_string()
                });
            }
            if model.model_name.namespace_is("packy") {
                return Ok(ServiceTarget {
                    endpoint: Endpoint::from_static("https://www.packyapi.com/v1/"),
                    model,
                    auth,
                });
            }
            Ok(ServiceTarget {
                endpoint,
                auth,
                model,
            })
        });

        let auth_resolver = AuthResolver::ResolverFn(Arc::new(Box::new(ConstantAuthResolver(key))));
        genai::Client::builder()
            .with_auth_resolver(auth_resolver)
            .with_service_target_resolver(target_resolver)
            .build()
    }
}

pub trait AvailabilityTest {
    async fn test_availability<S: AsRef<str>>(&self, model_name: S) -> Result<bool, genai::Error>;
}

impl AvailabilityTest for genai::Client {
    async fn test_availability<S: AsRef<str>>(&self, model_name: S) -> Result<bool, genai::Error> {
        let chat = ChatRequest::new(vec![ChatMessage::user(MessageContent::from_text(
            "Test your functionality. Reply with 'OK', no more.",
        ))]);
        let response = self.exec_chat(model_name.as_ref(), chat, None).await?;
        let response = response.texts();
        Ok(response.len() == 1 && response[0] == "OK")
    }
}
