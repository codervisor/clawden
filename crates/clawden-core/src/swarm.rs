use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmRole {
    Leader,
    Worker,
    Reviewer,
}

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

    pub fn create_team(&mut self, name: String, members: Vec<SwarmMember>) -> &SwarmTeam {
        self.teams.push(SwarmTeam { name, members });
        self.teams.last().expect("team just pushed")
    }

    pub fn list_teams(&self) -> &[SwarmTeam] {
        &self.teams
    }

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

    pub fn list_tasks(&self, parent: Option<&str>) -> Vec<&SwarmTask> {
        self.tasks
            .iter()
            .filter(|t| match parent {
                Some(pid) => t.parent_task.as_deref() == Some(pid),
                None => true,
            })
            .collect()
    }
}
