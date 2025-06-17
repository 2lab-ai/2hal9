//! Model Context Protocol (MCP) integration for 2HAL9
//!
//! This module implements MCP to provide standardized communication
//! between the wrapper server and individual neurons.

pub mod client;
pub mod protocol;
pub mod server;
pub mod tools;

pub use client::{MCPClient, WrapperMCPClient};
pub use protocol::{MCPError, MCPMessage, MCPRequest, MCPResponse};
pub use server::{MCPServer, NeuronMCPServer};
pub use tools::{
    FilesystemReadTool, FilesystemWriteTool, ProcessTaskTool, ShellTool, StatusTool, Tool,
    ToolContent, ToolDefinition, ToolRegistry, ToolResult, WebFetchTool,
};
