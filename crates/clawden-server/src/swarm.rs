use serde::{Deserialize, Serialize};

/// Role of an agent within a swarm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmRole {
    Leader,
    Worker,
    Reviewer,
}

/// A team definition: a named group of agents with assigned roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTeam {
    pub name: String,
    pub members: Vec<SwarmMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMember {
    pub agent_id: String,
    pub role: SwarmRole,
}

/// A task that has been decomposed for fan-out to a swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTask {
    pub id: String,
    pub parent_task: Option<String>,
    pub description: String,
    pub assigned_to: String,
    pub status: SwarmTaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmTaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Manages swarm teams and coordinates distributed tasks.
#[derive(Default)]
pub struct SwarmCoordinator {
    teams: Vec<SwarmTeam>,
    tasks: Vec<SwarmTask>,
    next_task_id: u64,
}

impl SwarmCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a new swarm team.
    pub fn create_team(&mut self, name: String, members: Vec<SwarmMember>) -> &SwarmTeam {
        self.teams.push(SwarmTeam { name, members });
        self.teams.last().unwrap()
    }

    /// List all teams.
    pub fn list_teams(&self) -> &[SwarmTeam] {
        &self.teams
    }

    /// Decompose a task into subtasks and assign to team members.
    pub fn fan_out(
        &mut self,
        team_name: &str,
        task_description: &str,
        subtask_descriptions: Vec<String>,
    ) -> Result<Vec<&SwarmTask>, String> {
        let team = self
            .teams
            .iter()
            .find(|t| t.name == team_name)
            .ok_or_else(|| format!("team '{team_name}' not found"))?;

        let workers: Vec<_> = team
            .members
            .iter()
            .filter(|m| m.role == SwarmRole::Worker || m.role == SwarmRole::Leader)
            .collect();

        if workers.is_empty() {
            return Err("team has no workers".to_string());
        }

        let parent_id = format!("swarm-task-{}", self.next_task_id);
        self.next_task_id += 1;

        // Create parent task assigned to leader (or first worker)
        let leader = team
            .members
            .iter()
            .find(|m| m.role == SwarmRole::Leader)
            .unwrap_or(workers[0]);

        self.tasks.push(SwarmTask {
            id: parent_id.clone(),
            parent_task: None,
            description: task_description.to_string(),
            assigned_to: leader.agent_id.clone(),
            status: SwarmTaskStatus::InProgress,
        });

        // Fan out subtasks to workers round-robin
        let start_idx = self.tasks.len();
        for (i, desc) in subtask_descriptions.iter().enumerate() {
            let worker = &workers[i % workers.len()];
            let task_id = format!("swarm-task-{}", self.next_task_id);
            self.next_task_id += 1;
            self.tasks.push(SwarmTask {
                id: task_id,
                parent_task: Some(parent_id.clone()),
                description: desc.clone(),
                assigned_to: worker.agent_id.clone(),
                status: SwarmTaskStatus::Pending,
            });
        }

        Ok(self.tasks[start_idx..].iter().collect())
    }

    /// Mark a task as completed.
    #[allow(dead_code)]
    pub fn complete_task(&mut self, task_id: &str) -> Result<(), String> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("task '{task_id}' not found"))?;
        task.status = SwarmTaskStatus::Completed;
        Ok(())
    }

    /// Get all tasks, optionally filtered to a parent.
    pub fn list_tasks(&self, parent: Option<&str>) -> Vec<&SwarmTask> {
        self.tasks
            .iter()
            .filter(|t| match parent {
                Some(pid) => t.parent_task.as_deref() == Some(pid),
                None => true,
            })
            .collect()
    }

    /// Check if all subtasks of a parent are completed.
    #[allow(dead_code)]
    pub fn is_fan_out_complete(&self, parent_id: &str) -> bool {
        let subtasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|t| t.parent_task.as_deref() == Some(parent_id))
            .collect();

        !subtasks.is_empty()
            && subtasks
                .iter()
                .all(|t| t.status == SwarmTaskStatus::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_team_and_fan_out() {
        let mut coord = SwarmCoordinator::new();
        coord.create_team(
            "alpha-team".to_string(),
            vec![
                SwarmMember {
                    agent_id: "agent-1".to_string(),
                    role: SwarmRole::Leader,
                },
                SwarmMember {
                    agent_id: "agent-2".to_string(),
                    role: SwarmRole::Worker,
                },
            ],
        );

        let subtasks = coord
            .fan_out(
                "alpha-team",
                "parent task",
                vec!["sub-1".to_string(), "sub-2".to_string()],
            )
            .unwrap();
        assert_eq!(subtasks.len(), 2);
        assert_eq!(subtasks[0].status, SwarmTaskStatus::Pending);
    }

    #[test]
    fn fan_out_completion_tracking() {
        let mut coord = SwarmCoordinator::new();
        coord.create_team(
            "team".to_string(),
            vec![SwarmMember {
                agent_id: "agent-1".to_string(),
                role: SwarmRole::Leader,
            }],
        );

        coord
            .fan_out("team", "root", vec!["sub".to_string()])
            .unwrap();

        let parent_id = "swarm-task-0";
        assert!(!coord.is_fan_out_complete(parent_id));

        coord.complete_task("swarm-task-1").unwrap();
        assert!(coord.is_fan_out_complete(parent_id));
    }
}
