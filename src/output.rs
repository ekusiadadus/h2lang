//! Output data structures for JSON serialization.

use crate::error::CompileError;
use crate::expander::Command;
use crate::scheduler::{AgentCommand, TimelineStep};
use serde::{Deserialize, Serialize};

/// Command type for JSON output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    /// Move straight (forward)
    Straight,
    /// Rotate right (90° clockwise)
    RotateRight,
    /// Rotate left (90° counter-clockwise)
    RotateLeft,
    /// Wait (no command for this step)
    Wait,
}

impl From<Command> for CommandType {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Straight => CommandType::Straight,
            Command::Right => CommandType::RotateRight,
            Command::Left => CommandType::RotateLeft,
        }
    }
}

/// toio command with optional parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToioCommand {
    /// Command type
    #[serde(rename = "type")]
    pub command_type: CommandType,

    /// Number of steps for straight movement (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,

    /// Rotation angle in degrees (default: 90)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<i32>,
}

impl ToioCommand {
    /// Create a straight command.
    pub fn straight() -> Self {
        Self {
            command_type: CommandType::Straight,
            steps: Some(1),
            angle: None,
        }
    }

    /// Create a rotate right command.
    pub fn rotate_right() -> Self {
        Self {
            command_type: CommandType::RotateRight,
            steps: None,
            angle: Some(90),
        }
    }

    /// Create a rotate left command.
    pub fn rotate_left() -> Self {
        Self {
            command_type: CommandType::RotateLeft,
            steps: None,
            angle: Some(-90),
        }
    }

    /// Create a wait command.
    pub fn wait() -> Self {
        Self {
            command_type: CommandType::Wait,
            steps: None,
            angle: None,
        }
    }
}

impl From<Command> for ToioCommand {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Straight => ToioCommand::straight(),
            Command::Right => ToioCommand::rotate_right(),
            Command::Left => ToioCommand::rotate_left(),
        }
    }
}

/// Compiled agent with command list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledAgent {
    /// Agent ID
    pub id: u32,
    /// List of commands
    pub commands: Vec<ToioCommand>,
}

/// Timeline entry for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Step number (0-based)
    pub step: usize,
    /// Commands for all agents at this step
    pub agent_commands: Vec<AgentTimelineCommand>,
}

/// Agent command in timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTimelineCommand {
    /// Agent ID
    pub agent_id: u32,
    /// Command to execute
    pub command: ToioCommand,
}

impl From<&AgentCommand> for AgentTimelineCommand {
    fn from(ac: &AgentCommand) -> Self {
        Self {
            agent_id: ac.agent_id,
            command: ToioCommand::from(ac.command),
        }
    }
}

impl From<&TimelineStep> for TimelineEntry {
    fn from(ts: &TimelineStep) -> Self {
        Self {
            step: ts.step,
            agent_commands: ts.agent_commands.iter().map(|ac| ac.into()).collect(),
        }
    }
}

/// Compiled program with all agents and timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProgram {
    /// List of compiled agents
    pub agents: Vec<CompiledAgent>,
    /// Maximum number of steps
    pub max_steps: usize,
    /// Execution timeline
    pub timeline: Vec<TimelineEntry>,
}

impl CompiledProgram {
    /// Create from expanded agents and timeline.
    pub fn from_expanded(expanded: &[(u32, Vec<Command>)], timeline: Vec<TimelineStep>) -> Self {
        let agents: Vec<CompiledAgent> = expanded
            .iter()
            .map(|(id, cmds)| CompiledAgent {
                id: *id,
                commands: cmds.iter().map(|c| ToioCommand::from(*c)).collect(),
            })
            .collect();

        let max_steps = timeline.len();
        let timeline: Vec<TimelineEntry> = timeline.iter().map(|ts| ts.into()).collect();

        Self {
            agents,
            max_steps,
            timeline,
        }
    }
}

/// Compile result (success or error).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum CompileResult {
    /// Successful compilation
    #[serde(rename = "success")]
    Success {
        /// Compiled program
        program: CompiledProgram,
    },
    /// Compilation error
    #[serde(rename = "error")]
    Error {
        /// List of errors
        errors: Vec<CompileError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_serialization() {
        let cmd = ToioCommand::straight();
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("straight"));
    }

    #[test]
    fn test_compile_result_success() {
        let program = CompiledProgram {
            agents: vec![],
            max_steps: 0,
            timeline: vec![],
        };
        let result = CompileResult::Success { program };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("success"));
    }

    #[test]
    fn test_compile_result_error() {
        let result = CompileResult::Error {
            errors: vec![CompileError {
                line: 1,
                column: 5,
                message: "Test error".to_string(),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
    }
}
