use std::sync::Mutex;

use crate::{
    component::webgpu::pointer_events::{HostPointerUp, PointerEvent, PointerUp, Pollable},
    HostEvent, HostState,
};
use tokio::sync::broadcast::Receiver;
use wasmtime::component::Resource;
use wasmtime_wasi::preview2::{self, WasiView};

#[async_trait::async_trait]
impl crate::component::webgpu::pointer_events::Host for HostState {
    async fn up(&mut self) -> wasmtime::Result<wasmtime::component::Resource<PointerUp>> {
        let receiver = self.sender.subscribe();
        let g = self
            .table_mut()
            .push(HostPointerEvent {
                receiver,
                data: Default::default(),
            })
            .unwrap();
        Ok(Resource::new_own(g.rep()))
    }
}

#[async_trait::async_trait]
impl HostPointerUp for HostState {
    async fn subscribe(
        &mut self,
        self_: Resource<PointerUp>,
    ) -> wasmtime::Result<Resource<Pollable>> {
        let g: Resource<HostPointerEvent> = Resource::new_own(self_.rep());
        let gg = preview2::subscribe(self.table_mut(), g).unwrap();
        Ok(gg)
    }
    async fn get(&mut self, self_: Resource<PointerUp>) -> wasmtime::Result<Option<PointerEvent>> {
        let g: Resource<HostPointerEvent> = Resource::new_own(self_.rep());
        let ddd = self.table.get(&g).unwrap();
        let res = ddd.data.lock().unwrap().take();
        Ok(res)
    }
    fn drop(&mut self, _self_: Resource<PointerUp>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct HostPointerEvent {
    receiver: Receiver<HostEvent>,
    data: Mutex<Option<PointerEvent>>,
}

#[async_trait::async_trait]
impl preview2::Subscribe for HostPointerEvent {
    async fn ready(&mut self) {
        loop {
            let event = self.receiver.recv().await.unwrap();
            if let HostEvent::PointerEvent { x, y } = event {
                *self.data.lock().unwrap() = Some(PointerEvent { x, y });
                return;
            }
        }
    }
}