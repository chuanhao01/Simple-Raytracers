pub struct PathTracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl PathTracer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        device.on_uncaptured_error(Box::new(|error| panic!("Aborting due to error, {}", error)));

        // TODO: init GPU resources

        Self { device, queue }
    }
}
