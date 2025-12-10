//! Parallel scheduling for multiple agents.

use crate::expander::Command;

/// A command for a specific agent at a specific step.
#[derive(Debug, Clone)]
pub struct AgentCommand {
    /// Agent ID
    pub agent_id: u32,
    /// Command to execute
    pub command: Command,
}

/// A single step in the timeline (all agents' commands at this step).
#[derive(Debug, Clone)]
pub struct TimelineStep {
    /// Step number (0-based)
    pub step: usize,
    /// Commands for all agents at this step
    pub agent_commands: Vec<AgentCommand>,
}

/// Scheduler for parallel execution of multiple agents.
pub struct Scheduler;

impl Scheduler {
    /// Schedule multiple agents' commands into a timeline.
    ///
    /// Each step in the timeline contains all agents' commands for that step.
    /// If an agent has fewer commands than the maximum, it won't have a command
    /// for the remaining steps (wait state).
    pub fn schedule(agents: &[(u32, Vec<Command>)]) -> Vec<TimelineStep> {
        if agents.is_empty() {
            return Vec::new();
        }

        // Find maximum number of steps
        let max_len = agents.iter().map(|(_, cmds)| cmds.len()).max().unwrap_or(0);

        let mut timeline = Vec::with_capacity(max_len);

        for step in 0..max_len {
            let mut agent_commands = Vec::new();

            for (agent_id, commands) in agents {
                if let Some(cmd) = commands.get(step) {
                    agent_commands.push(AgentCommand {
                        agent_id: *agent_id,
                        command: *cmd,
                    });
                }
                // If no command for this step, agent waits (not included in timeline)
            }

            timeline.push(TimelineStep {
                step,
                agent_commands,
            });
        }

        timeline
    }

    /// Get the maximum number of steps across all agents.
    pub fn max_steps(agents: &[(u32, Vec<Command>)]) -> usize {
        agents.iter().map(|(_, cmds)| cmds.len()).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_agent() {
        let agents = vec![(0, vec![Command::Straight, Command::Right, Command::Left])];

        let timeline = Scheduler::schedule(&agents);

        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].agent_commands.len(), 1);
        assert_eq!(timeline[0].agent_commands[0].agent_id, 0);
        assert_eq!(timeline[0].agent_commands[0].command, Command::Straight);
    }

    #[test]
    fn test_multiple_agents_same_length() {
        let agents = vec![
            (0, vec![Command::Straight, Command::Right]),
            (1, vec![Command::Left, Command::Straight]),
        ];

        let timeline = Scheduler::schedule(&agents);

        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].agent_commands.len(), 2);
        assert_eq!(timeline[1].agent_commands.len(), 2);
    }

    #[test]
    fn test_multiple_agents_different_length() {
        let agents = vec![
            (0, vec![Command::Straight, Command::Right, Command::Left]),
            (1, vec![Command::Left]),
        ];

        let timeline = Scheduler::schedule(&agents);

        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].agent_commands.len(), 2);
        assert_eq!(timeline[1].agent_commands.len(), 1); // Agent 1 is done
        assert_eq!(timeline[2].agent_commands.len(), 1); // Only agent 0
    }

    #[test]
    fn test_empty_agents() {
        let agents: Vec<(u32, Vec<Command>)> = vec![];
        let timeline = Scheduler::schedule(&agents);
        assert!(timeline.is_empty());
    }

    #[test]
    fn test_max_steps() {
        let agents = vec![
            (0, vec![Command::Straight, Command::Right, Command::Left]),
            (1, vec![Command::Left]),
        ];

        assert_eq!(Scheduler::max_steps(&agents), 3);
    }
}
