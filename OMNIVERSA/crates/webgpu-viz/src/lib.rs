//! WebGPU Visualization — 4D Manifold Rendering
//! 
//! Renders the Phase Space Manifold as an interactive 4D visualization,
//! allowing users to "fly" through the liquidity field.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use nalgebra::{Vector3, Matrix4, Perspective3};
use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};

pub mod renderer;
pub mod shaders;
pub mod camera;

pub use renderer::{ManifoldRenderer, RenderConfig};
pub use camera::{OrbitalCamera, CameraController};

/// Vertex for the manifold mesh.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ManifoldVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub stress: f32,
    pub viscosity: f32,
}

/// Uniforms for the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShaderUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub time: f32,
    pub fragility: f32,
    pub _padding: [f32; 2],
}

/// A renderable frame of the manifold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldFrame {
    pub vertices: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub fragility_score: f32,
    pub timestamp_ns: u64,
}
