use crate::interface::DeviceService;
use crate::utils::platform::{DeviceKind, PlatformInfo};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct DeviceNode {
    pub id: usize,
    pub compatible: String,
    pub base_addr: usize,
    pub size: usize,
    pub irq: u32,
    pub kind: DeviceKind,
    pub parent_id: Option<usize>,
    pub children: Vec<usize>,
}

impl DeviceNode {
    pub fn compatible_str(&self) -> &str {
        &self.compatible
    }
}

pub struct DeviceManager {
    nodes: Vec<DeviceNode>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: DeviceNode) {
        self.nodes.push(node);
    }

    pub fn get_roots(&self) -> Vec<&DeviceNode> {
        self.nodes.iter().filter(|n| n.parent_id.is_none()).collect()
    }

    pub fn print_tree(&self) {
        let roots = self.get_roots();
        for root in roots {
            self.print_node(root, 0);
        }
    }

    fn print_node(&self, node: &DeviceNode, depth: usize) {
        crate::println!(
            "{:indent$}- [{}] {} @ {:#x}",
            "",
            node.id,
            node.compatible_str(),
            node.base_addr,
            indent = depth * 2
        );
        for &child_id in &node.children {
            if let Some(child) = self.get_node(child_id) {
                self.print_node(child, depth + 1);
            }
        }
    }
}

impl DeviceService for DeviceManager {
    fn get_node(&self, id: usize) -> Option<&DeviceNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    fn find_compatible(&self, compat: &str) -> Option<&DeviceNode> {
        self.nodes.iter().find(|n| n.compatible_str().contains(compat))
    }

    fn scan_platform(&mut self, info: &PlatformInfo) {
        let mut nodes = Vec::new();

        // First pass: Create nodes
        for (i, dev_desc) in info.devices[..info.device_count].iter().enumerate() {
            let parent_id = if dev_desc.parent_index == u32::MAX {
                None
            } else {
                Some(dev_desc.parent_index as usize)
            };

            let compat_len = dev_desc
                .compatible
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(dev_desc.compatible.len());
            let compat_str =
                core::str::from_utf8(&dev_desc.compatible[..compat_len]).unwrap_or("???");

            let node = DeviceNode {
                id: i,
                compatible: String::from(compat_str),
                base_addr: dev_desc.base_addr,
                size: dev_desc.size,
                irq: dev_desc.irq,
                kind: dev_desc.kind,
                parent_id,
                children: Vec::new(),
            };
            nodes.push(node);
        }

        // Second pass: Build children relationships
        for i in 0..nodes.len() {
            if let Some(pid) = nodes[i].parent_id {
                if pid < nodes.len() {
                    nodes[pid].children.push(i);
                }
            }
        }

        self.nodes = nodes;
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub id: usize,
    pub dev_type: DeviceKind,
}
