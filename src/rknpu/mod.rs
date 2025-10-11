use alloc::vec::Vec;

use rdrive::{
    DriverGeneric, PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo,
};
use rknpu::{Rknpu, RknpuConfig, RknpuType};

use crate::iomap;

module_driver!(
    name: "Rockchip NPU",
    level: ProbeLevel::PostKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["rockchip,rk3588-rknpu"],
            on_probe: probe
        }
    ],
);

fn probe(info: FdtInfo<'_>, plat_dev: PlatformDevice) -> Result<(), OnProbeError> {
    let mut config = None;
    for c in info.node.compatibles() {
        if c == "rockchip,rk3588-rknpu" {
            config = Some(RknpuConfig {
                rknpu_type: RknpuType::Rk3588,
            });
            break;
        }
    }

    let config = config.expect("Unsupported RKNPU compatible");
    let regs = info.node.reg().unwrap();

    let mut base_regs = Vec::new();
    let page_size = 0x1000;
    for reg in regs {
        let start_raw = reg.address as usize;
        let end = start_raw + reg.size.unwrap_or(0x1000);

        let start = start_raw & !(page_size - 1);
        let offset = start_raw - start;
        let end = (end + page_size - 1) & !(page_size - 1);
        let size = end - start;

        base_regs.push(unsafe { iomap(start as _, size)?.add(offset) });
    }

    let npu = Rknpu::new(&base_regs, config);
    plat_dev.register(npu);
    info!("NPU registered successfully");
    Ok(())
}
