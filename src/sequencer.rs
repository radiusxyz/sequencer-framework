use std::{any, future::Future, mem::MaybeUninit, path::Path, sync::Once};

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    hyper::{header, Method},
    jsonrpsee::server::{middleware::http::ProxyGetRequestLayer, RpcModule, Server, ServerHandle},
    serde::Serialize,
    serde_json,
    tokio::{
        runtime::{Builder, Runtime},
        task::JoinHandle,
    },
    tower::ServiceBuilder,
    tower_http::cors::{Any, CorsLayer},
    tracing_subscriber, unrecoverable,
};
use sequencer_database::Database;
use sequencer_json_rpc::method::RpcMethod;

static mut SEQUENCER: MaybeUninit<Sequencer> = MaybeUninit::uninit();
static INIT: Once = Once::new();

pub(crate) fn sequencer() -> &'static Sequencer {
    if INIT.is_completed() {
        unsafe { SEQUENCER.assume_init_ref() }
    } else {
        unrecoverable!("Sequencer has not been initialized")
    }
}

pub struct SequencerBuilder {
    database: Database,
    thread_count: usize,
    rpc_endpoint: String,
    rpc_module: RpcModule<Database>,
}

impl SequencerBuilder {
    pub fn new(
        thread_count: usize,
        database_path: impl AsRef<Path>,
        rpc_server_endpoint: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let database =
            Database::new(database_path.as_ref()).wrap(caller!(SequencerBuilder::new()))?;

        Ok(Self {
            database: database.clone(),
            thread_count,
            rpc_endpoint: rpc_server_endpoint.as_ref().into(),
            rpc_module: RpcModule::new(database),
        })
    }

    pub fn register_rpc_method<T>(mut self) -> Result<Self, Error>
    where
        T: RpcMethod,
        T::Response: Clone + Serialize + 'static,
    {
        self.rpc_module
            .register_async_method(T::method_name(), |parameter, state| async move {
                let rpc_parameter: T = parameter.parse().wrap_context(
                    caller!(RpcMethod::handler()),
                    format_args!("{:#?}", parameter),
                )?;
                rpc_parameter.handler(state).await
            })
            .wrap_context(
                caller!(SequencerBuilder::register_rpc_method()),
                format_args!("parameter: {:?}", any::type_name::<T>()),
            )?;
        Ok(self)
    }

    pub fn build(self) -> Result<Sequencer, Error> {
        Sequencer::build(self)
    }
}

pub struct Sequencer {
    database: Database,
    runtime: Runtime,
    rpc_server_handle: ServerHandle,
}

impl Sequencer {
    async fn build_rpc_server(
        rpc_endpoint: String,
        mut rpc_module: RpcModule<Database>,
    ) -> Result<ServerHandle, Error> {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);

        let middleware = ServiceBuilder::new().layer(cors).layer(
            ProxyGetRequestLayer::new("/health", "system_health")
                .wrap(caller!(Sequencer::build_rpc_server()))?,
        );

        let rpc_server = Server::builder()
            .set_http_middleware(middleware)
            .build(rpc_endpoint)
            .await
            .wrap(caller!(Sequencer::build_rpc_server()))?;

        rpc_module
            .register_method("health_check", |_, _| serde_json::json!({ "health": true }))
            .wrap_context(
                caller!(Sequencer::build_rpc_server()),
                "Failed to register 'health_check' endpoint",
            )?;

        Ok(rpc_server.start(rpc_module))
    }

    pub fn build(builder: SequencerBuilder) -> Result<Self, Error> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(builder.thread_count)
            .build()
            .wrap_context(
                caller!(Sequencer::build()),
                "Failed to initialize tokio runtime",
            )?;

        let rpc_server_handle = runtime
            .block_on(Self::build_rpc_server(
                builder.rpc_endpoint,
                builder.rpc_module,
            ))
            .wrap_context(
                caller!(Sequencer::build()),
                "Failed to initialize the RPC server",
            )?;

        Ok(Self {
            database: builder.database,
            runtime,
            rpc_server_handle,
        })
    }

    pub fn init(self) {
        unsafe {
            INIT.call_once(|| {
                tracing_subscriber::fmt().init();
                SEQUENCER.write(self);
            });
        }

        // sequencer().block_on(async move { while let false = sequencer().is_stopped() {} });
    }

    pub fn is_stopped(&self) -> bool {
        self.rpc_server_handle.is_stopped()
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.runtime.block_on(future)
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}
