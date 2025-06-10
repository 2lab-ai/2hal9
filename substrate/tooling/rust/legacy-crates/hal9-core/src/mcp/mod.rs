//! Model Context Protocol (MCP) integration for 2HAL9
//! 
//! This module implements MCP to provide standardized communication
//! between the wrapper server and individual neurons.

pub mod protocol;
pub mod server;
pub mod client;
pub mod tools;

pub use protocol::{MCPMessage, MCPRequest, MCPResponse, MCPError};
pub use server::{MCPServer, NeuronMCPServer};
pub use client::{MCPClient, WrapperMCPClient};
pub use tools::{
    Tool, ToolDefinition, ToolResult, ToolContent, ToolRegistry,
    ProcessTaskTool, StatusTool,
    FilesystemReadTool, FilesystemWriteTool, ShellTool, WebFetchTool
};