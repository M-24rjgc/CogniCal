use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::dependency::{
    DependencyCreateInput, DependencyEdge, DependencyFilter, DependencyGraph, DependencyType,
    DependencyValidation, ReadyTask, TaskDependency, TaskNode,
};
use rusqlite::params;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct DependencyService {
    db_pool: DbPool,
    graph_cache: Arc<RwLock<Option<DependencyGraph>>>,
    cache_timestamp: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl DependencyService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            graph_cache: Arc::new(RwLock::new(None)),
            cache_timestamp: Arc::new(RwLock::new(None)),
        }
    }

    /// Invalidate the graph cache
    fn invalidate_cache(&self) {
        if let Ok(mut cache) = self.graph_cache.write() {
            *cache = None;
        }
        if let Ok(mut timestamp) = self.cache_timestamp.write() {
            *timestamp = None;
        }
    }

    /// Check if cache is valid (less than 5 minutes old)
    fn is_cache_valid(&self) -> bool {
        if let Ok(timestamp) = self.cache_timestamp.read() {
            if let Some(ts) = *timestamp {
                let now = chrono::Utc::now();
                let cache_age = now.signed_duration_since(ts);
                return cache_age.num_minutes() < 5;
            }
        }
        false
    }

    /// Add a new dependency relationship between tasks
    pub async fn add_dependency(&self, input: DependencyCreateInput) -> AppResult<String> {
        // Validate the dependency first
        let validation = self
            .validate_dependency(&input.predecessor_id, &input.successor_id)
            .await?;

        if !validation.is_valid {
            return Err(crate::error::AppError::validation(
                validation
                    .error_message
                    .unwrap_or_else(|| "Invalid dependency".to_string()),
            ));
        }

        let dependency_id = Uuid::new_v4().to_string();
        let dependency_type = input.dependency_type.unwrap_or_default();
        let now = chrono::Utc::now().to_rfc3339();

        let conn = self.db_pool.get_connection()?;
        conn.execute(
            "INSERT INTO task_dependencies (id, predecessor_id, successor_id, dependency_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                dependency_id,
                input.predecessor_id,
                input.successor_id,
                dependency_type.to_string(),
                now
            ],
        )?;

        // Invalidate cache after modification
        self.invalidate_cache();

        Ok(dependency_id)
    }

    /// Remove a dependency relationship
    pub async fn remove_dependency(&self, dependency_id: &str) -> AppResult<()> {
        let conn = self.db_pool.get_connection()?;
        let rows_affected = conn.execute(
            "DELETE FROM task_dependencies WHERE id = ?1",
            params![dependency_id],
        )?;

        if rows_affected == 0 {
            return Err(crate::error::AppError::not_found());
        }

        // Invalidate cache after modification
        self.invalidate_cache();

        Ok(())
    }

    /// Update the type of a dependency relationship
    pub async fn update_dependency_type(
        &self,
        dependency_id: &str,
        dependency_type: DependencyType,
    ) -> AppResult<()> {
        let conn = self.db_pool.get_connection()?;
        let rows_affected = conn.execute(
            "UPDATE task_dependencies SET dependency_type = ?1 WHERE id = ?2",
            params![dependency_type.to_string(), dependency_id],
        )?;

        if rows_affected == 0 {
            return Err(crate::error::AppError::not_found());
        }

        // Invalidate cache after modification
        self.invalidate_cache();

        Ok(())
    }

    /// Validate a potential dependency relationship
    pub async fn validate_dependency(
        &self,
        predecessor_id: &str,
        successor_id: &str,
    ) -> AppResult<DependencyValidation> {
        // Check if tasks exist
        let conn = self.db_pool.get_connection()?;

        let predecessor_exists: bool = conn
            .prepare("SELECT 1 FROM tasks WHERE id = ?1")?
            .exists(params![predecessor_id])?;

        let successor_exists: bool = conn
            .prepare("SELECT 1 FROM tasks WHERE id = ?1")?
            .exists(params![successor_id])?;

        if !predecessor_exists {
            return Ok(DependencyValidation {
                is_valid: false,
                error_message: Some(format!("Predecessor task {} not found", predecessor_id)),
                would_create_cycle: false,
                cycle_path: None,
            });
        }

        if !successor_exists {
            return Ok(DependencyValidation {
                is_valid: false,
                error_message: Some(format!("Successor task {} not found", successor_id)),
                would_create_cycle: false,
                cycle_path: None,
            });
        }

        // Check if dependency already exists
        let dependency_exists: bool = conn
            .prepare(
                "SELECT 1 FROM task_dependencies WHERE predecessor_id = ?1 AND successor_id = ?2",
            )?
            .exists(params![predecessor_id, successor_id])?;

        if dependency_exists {
            return Ok(DependencyValidation {
                is_valid: false,
                error_message: Some("Dependency already exists".to_string()),
                would_create_cycle: false,
                cycle_path: None,
            });
        }

        // Check for self-dependency
        if predecessor_id == successor_id {
            return Ok(DependencyValidation {
                is_valid: false,
                error_message: Some("Task cannot depend on itself".to_string()),
                would_create_cycle: true,
                cycle_path: Some(vec![predecessor_id.to_string()]),
            });
        }

        // Check if adding this dependency would create a cycle
        let graph = self.get_dependency_graph(None).await?;
        let cycle_path = self.detect_cycle_with_new_edge(&graph, predecessor_id, successor_id);

        if let Some(path) = cycle_path {
            return Ok(DependencyValidation {
                is_valid: false,
                error_message: Some(
                    "Adding this dependency would create a circular dependency".to_string(),
                ),
                would_create_cycle: true,
                cycle_path: Some(path),
            });
        }

        Ok(DependencyValidation {
            is_valid: true,
            error_message: None,
            would_create_cycle: false,
            cycle_path: None,
        })
    }

    /// Get all dependencies for a specific task
    pub async fn get_task_dependencies(&self, task_id: &str) -> AppResult<Vec<TaskDependency>> {
        let conn = self.db_pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, predecessor_id, successor_id, dependency_type, created_at
             FROM task_dependencies
             WHERE predecessor_id = ?1 OR successor_id = ?1
             ORDER BY created_at DESC",
        )?;

        let dependencies = stmt
            .query_map(params![task_id], |row| {
                Ok(TaskDependency {
                    id: row.get(0)?,
                    predecessor_id: row.get(1)?,
                    successor_id: row.get(2)?,
                    dependency_type: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(dependencies)
    }

    /// Get all dependencies in the system
    pub async fn get_all_dependencies(&self) -> AppResult<Vec<TaskDependency>> {
        let conn = self.db_pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, predecessor_id, successor_id, dependency_type, created_at
             FROM task_dependencies
             ORDER BY created_at DESC",
        )?;

        let dependencies = stmt
            .query_map([], |row| {
                Ok(TaskDependency {
                    id: row.get(0)?,
                    predecessor_id: row.get(1)?,
                    successor_id: row.get(2)?,
                    dependency_type: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(dependencies)
    }

    /// Get a dependency by ID
    pub async fn get_dependency_by_id(
        &self,
        dependency_id: &str,
    ) -> AppResult<Option<TaskDependency>> {
        let conn = self.db_pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, predecessor_id, successor_id, dependency_type, created_at
             FROM task_dependencies
             WHERE id = ?1",
        )?;

        match stmt.query_row(params![dependency_id], |row| {
            Ok(TaskDependency {
                id: row.get(0)?,
                predecessor_id: row.get(1)?,
                successor_id: row.get(2)?,
                dependency_type: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                created_at: row.get(4)?,
            })
        }) {
            Ok(dependency) => Ok(Some(dependency)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Get tasks that are ready to start (no incomplete dependencies)
    pub async fn get_ready_tasks(&self) -> AppResult<Vec<ReadyTask>> {
        let conn = self.db_pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, status, priority, due_at FROM ready_tasks ORDER BY priority DESC, due_at ASC"
        )?;

        let ready_tasks = stmt
            .query_map([], |row| {
                Ok(ReadyTask {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    status: row.get(2)?,
                    priority: row.get(3)?,
                    due_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ready_tasks)
    }

    /// Calculate critical path for a specific goal task
    pub async fn calculate_critical_path(&self, goal_task_id: &str) -> AppResult<Vec<String>> {
        let graph = self.get_dependency_graph(None).await?;

        if !graph.nodes.contains_key(goal_task_id) {
            return Err(crate::error::AppError::not_found());
        }

        let critical_path =
            self.find_critical_path_to_goal(&graph.nodes, &graph.edges, goal_task_id)?;
        Ok(critical_path)
    }

    /// Get the complete dependency graph with caching
    pub async fn get_dependency_graph(
        &self,
        filter: Option<DependencyFilter>,
    ) -> AppResult<DependencyGraph> {
        // For filtered requests, don't use cache
        if filter.is_some() {
            return self.build_dependency_graph(filter).await;
        }

        // Check if we have a valid cached graph
        if self.is_cache_valid() {
            if let Ok(cache) = self.graph_cache.read() {
                if let Some(ref cached_graph) = *cache {
                    return Ok(cached_graph.clone());
                }
            }
        }

        // Build new graph and cache it
        let graph = self.build_dependency_graph(filter).await?;

        // Update cache
        if let Ok(mut cache) = self.graph_cache.write() {
            *cache = Some(graph.clone());
        }
        if let Ok(mut timestamp) = self.cache_timestamp.write() {
            *timestamp = Some(chrono::Utc::now());
        }

        Ok(graph)
    }

    /// Build the dependency graph from database
    async fn build_dependency_graph(
        &self,
        filter: Option<DependencyFilter>,
    ) -> AppResult<DependencyGraph> {
        let conn = self.db_pool.get_connection()?;

        // Build task filter condition
        let (task_filter_sql, _include_completed) = if let Some(ref f) = filter {
            let mut conditions = Vec::new();

            if let Some(ref task_ids) = f.task_ids {
                let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                conditions.push(format!("t.id IN ({})", placeholders));
            }

            if !f.include_completed.unwrap_or(true) {
                conditions.push("t.status != 'completed'".to_string());
            }

            let filter_sql = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            (filter_sql, f.include_completed.unwrap_or(true))
        } else {
            (String::new(), true)
        };

        // Get all relevant tasks
        let task_query = format!(
            "SELECT id, status FROM tasks {} ORDER BY id",
            task_filter_sql
        );

        let mut stmt = conn.prepare(&task_query)?;
        let mut params_vec = Vec::new();

        if let Some(ref f) = filter {
            if let Some(ref task_ids) = f.task_ids {
                for task_id in task_ids {
                    params_vec.push(task_id.as_str());
                }
            }
        }

        let task_rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut nodes = HashMap::new();
        let task_statuses: HashMap<String, String> =
            task_rows.collect::<Result<HashMap<_, _>, _>>()?;

        // Initialize nodes
        for (task_id, status) in &task_statuses {
            nodes.insert(
                task_id.clone(),
                TaskNode {
                    task_id: task_id.clone(),
                    status: status.clone(),
                    dependencies: Vec::new(),
                    dependents: Vec::new(),
                    is_ready: false,
                },
            );
        }

        // Get all dependencies
        let mut stmt = conn.prepare(
            "SELECT id, predecessor_id, successor_id, dependency_type, created_at
             FROM task_dependencies
             ORDER BY created_at",
        )?;

        let dependency_rows = stmt.query_map([], |row| {
            Ok(TaskDependency {
                id: row.get(0)?,
                predecessor_id: row.get(1)?,
                successor_id: row.get(2)?,
                dependency_type: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                created_at: row.get(4)?,
            })
        })?;

        let mut edges = Vec::new();

        for dependency_result in dependency_rows {
            let dependency = dependency_result?;

            // Only include dependencies where both tasks are in our node set
            if nodes.contains_key(&dependency.predecessor_id)
                && nodes.contains_key(&dependency.successor_id)
            {
                // Update node relationships
                if let Some(predecessor_node) = nodes.get_mut(&dependency.predecessor_id) {
                    predecessor_node
                        .dependents
                        .push(dependency.successor_id.clone());
                }

                if let Some(successor_node) = nodes.get_mut(&dependency.successor_id) {
                    successor_node
                        .dependencies
                        .push(dependency.predecessor_id.clone());
                }

                // Add edge
                edges.push(DependencyEdge {
                    id: dependency.id,
                    source: dependency.predecessor_id,
                    target: dependency.successor_id,
                    dependency_type: dependency.dependency_type,
                });
            }
        }

        // Calculate ready status for each node
        for node in nodes.values_mut() {
            node.is_ready = node.dependencies.iter().all(|dep_id| {
                task_statuses
                    .get(dep_id)
                    .map_or(true, |status| status == "completed")
            });
        }

        // Calculate topological order
        let topological_order = self.topological_sort(&nodes, &edges)?;

        // Calculate critical path
        let critical_path =
            self.calculate_critical_path_internal(&nodes, &edges, &topological_order)?;

        Ok(DependencyGraph {
            nodes,
            edges,
            topological_order,
            critical_path,
        })
    }

    /// Detect if adding a new edge would create a cycle
    fn detect_cycle_with_new_edge(
        &self,
        graph: &DependencyGraph,
        new_predecessor: &str,
        new_successor: &str,
    ) -> Option<Vec<String>> {
        // Create a temporary graph with the new edge
        let mut temp_edges = graph.edges.clone();
        temp_edges.push(DependencyEdge {
            id: "temp".to_string(),
            source: new_predecessor.to_string(),
            target: new_successor.to_string(),
            dependency_type: DependencyType::FinishToStart,
        });

        // Build adjacency list
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for node_id in graph.nodes.keys() {
            adj_list.insert(node_id.clone(), Vec::new());
        }

        for edge in &temp_edges {
            adj_list
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }

        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                if let Some(cycle_path) = self.dfs_cycle_detection(
                    node_id,
                    &adj_list,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
                    return Some(cycle_path);
                }
            }
        }

        None
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        adj_list: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adj_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle_path) =
                        self.dfs_cycle_detection(neighbor, adj_list, visited, rec_stack, path)
                    {
                        return Some(cycle_path);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle - return the path from the cycle start
                    let cycle_start_idx = path.iter().position(|x| x == neighbor).unwrap();
                    let mut cycle_path = path[cycle_start_idx..].to_vec();
                    cycle_path.push(neighbor.clone()); // Close the cycle
                    return Some(cycle_path);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// Perform topological sorting on the dependency graph
    fn topological_sort(
        &self,
        nodes: &HashMap<String, TaskNode>,
        edges: &[DependencyEdge],
    ) -> AppResult<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for node_id in nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adj_list.insert(node_id.clone(), Vec::new());
        }

        // Build the graph
        for edge in edges {
            adj_list
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
        }

        // Kahn's algorithm for topological sorting
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Add all nodes with in-degree 0 to the queue
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // Reduce in-degree of neighbors
            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        // Check if all nodes are included (no cycles)
        if result.len() != nodes.len() {
            return Err(crate::error::AppError::validation(
                "Circular dependency detected in graph",
            ));
        }

        Ok(result)
    }

    /// Calculate the critical path for the entire graph (longest path)
    fn calculate_critical_path_internal(
        &self,
        nodes: &HashMap<String, TaskNode>,
        edges: &[DependencyEdge],
        topological_order: &[String],
    ) -> AppResult<Vec<String>> {
        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        // Build adjacency list for reverse traversal
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for node_id in nodes.keys() {
            adj_list.insert(node_id.clone(), Vec::new());
        }

        for edge in edges {
            adj_list
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        // Find the longest path using dynamic programming
        let mut distances: HashMap<String, i32> = HashMap::new();
        let mut predecessors: HashMap<String, Option<String>> = HashMap::new();

        // Initialize distances
        for node_id in nodes.keys() {
            distances.insert(node_id.clone(), 0);
            predecessors.insert(node_id.clone(), None);
        }

        // Process nodes in reverse topological order to find longest paths
        for node_id in topological_order.iter().rev() {
            if let Some(dependencies) = adj_list.get(node_id) {
                for dep_id in dependencies {
                    let new_distance = distances.get(node_id).unwrap_or(&0) + 1;
                    let current_distance = distances.get(dep_id).unwrap_or(&0);

                    if new_distance > *current_distance {
                        distances.insert(dep_id.clone(), new_distance);
                        predecessors.insert(dep_id.clone(), Some(node_id.clone()));
                    }
                }
            }
        }

        // Find the node with the maximum distance (end of critical path)
        let max_node = distances
            .iter()
            .max_by_key(|(_, &distance)| distance)
            .map(|(node_id, _)| node_id.clone());

        if let Some(end_node) = max_node {
            // Reconstruct the critical path
            let mut path = Vec::new();
            let mut current = Some(end_node);

            while let Some(node_id) = current {
                path.push(node_id.clone());
                current = predecessors.get(&node_id).and_then(|pred| pred.clone());
            }

            path.reverse();
            Ok(path)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find critical path to a specific goal task
    fn find_critical_path_to_goal(
        &self,
        nodes: &HashMap<String, TaskNode>,
        edges: &[DependencyEdge],
        goal_task_id: &str,
    ) -> AppResult<Vec<String>> {
        // Build adjacency list for reverse traversal (from goal backwards)
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for node_id in nodes.keys() {
            adj_list.insert(node_id.clone(), Vec::new());
        }

        for edge in edges {
            adj_list
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        // Use BFS to find all paths to the goal, then select the longest
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut distances: HashMap<String, i32> = HashMap::new();
        let mut predecessors: HashMap<String, Option<String>> = HashMap::new();

        // Start from the goal task
        queue.push_back(goal_task_id.to_string());
        distances.insert(goal_task_id.to_string(), 0);
        predecessors.insert(goal_task_id.to_string(), None);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(dependencies) = adj_list.get(&current) {
                for dep_id in dependencies {
                    let new_distance = distances.get(&current).unwrap_or(&0) + 1;
                    let current_distance = distances.get(dep_id).unwrap_or(&-1);

                    if new_distance > *current_distance {
                        distances.insert(dep_id.clone(), new_distance);
                        predecessors.insert(dep_id.clone(), Some(current.clone()));
                        queue.push_back(dep_id.clone());
                    }
                }
            }
        }

        // Find the starting node with maximum distance (beginning of critical path)
        let start_node = distances
            .iter()
            .filter(|(node_id, _)| {
                // Only consider nodes that have no dependencies (root nodes)
                !edges.iter().any(|edge| &edge.target == *node_id)
            })
            .max_by_key(|(_, &distance)| distance)
            .map(|(node_id, _)| node_id.clone());

        if let Some(start_node_id) = start_node {
            // Reconstruct the critical path from start to goal
            let mut path = Vec::new();
            let mut current = Some(start_node_id);

            // Build path from start to goal by following the reverse of predecessors
            while let Some(node_id) = current {
                path.push(node_id.clone());

                // Find the next node in the path (the one that has current as predecessor)
                current = predecessors
                    .iter()
                    .find(|(_, pred)| pred.as_ref() == Some(&node_id))
                    .map(|(next_node, _)| next_node.clone());

                if current.as_ref() == Some(&goal_task_id.to_string()) {
                    path.push(goal_task_id.to_string());
                    break;
                }
            }

            Ok(path)
        } else {
            // If no path found, return just the goal task
            Ok(vec![goal_task_id.to_string()])
        }
    }
}
