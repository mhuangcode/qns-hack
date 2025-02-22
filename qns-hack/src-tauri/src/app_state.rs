use serde::Serialize;
use std::sync::Arc;
use sysinfo::System;
use tauri::{window, App, AppHandle, Emitter, Manager, WebviewWindow};
use tokio::{runtime::Handle, sync::RwLock};

#[derive(Clone, Serialize)]
pub struct ProcessUpdatedEvent {
    pid: String,
    name: String,
    cpu: f32,
}

pub struct AppState {
    runtime: Arc<tokio::runtime::Runtime>,
    sys: Arc<RwLock<System>>,
    app_handle: AppHandle,
}

impl AppState {
    pub fn new(app: &mut App) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut sys = System::new_all();
        sys.refresh_all();

        let app_handle = app.handle();

        Self {
            runtime: Arc::new(runtime),
            sys: Arc::new(RwLock::new(sys)),
            app_handle: app_handle.clone(),
        }
    }

    pub async fn run(&self) {
        let sys = self.sys.clone();
        let app_handle = self.app_handle.clone();
        self.runtime.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                let mut sys = sys.write().await;
                interval.tick().await;
                sys.refresh_all();
                let process_info = sys.processes();
                for (pid, proc_) in process_info {
                    let name = proc_.name();

                    match app_handle.emit(
                        "proccess-updated",
                        ProcessUpdatedEvent {
                            pid: pid.to_string(),
                            name: name.to_os_string().into_string().unwrap(),
                            cpu: proc_.cpu_usage(),
                        },
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("failed to emit event: {:?}", e);
                        }
                    };
                }
            }
        });
    }

    pub fn runtime(&self) -> Handle {
        self.runtime.handle().clone()
    }
}
