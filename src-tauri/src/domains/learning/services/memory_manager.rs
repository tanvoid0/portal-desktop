use sea_orm::DatabaseConnection;
use crate::domains::learning::repositories::{
    LearnedPatternRepository, UserPreferenceRepository, LearningEventRepository,
};

// Import logger macros
use crate::{log_info, log_warn};

/// Memory and retention policies for learning data
pub struct MemoryManager {
    // Retention days
    event_retention_days: i64,
    pattern_retention_days: i64,
    // Limits
    max_events: usize,
    max_patterns_per_type: usize,
    max_preferences_per_type: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            // Default retention: keep events for 90 days, patterns forever
            event_retention_days: 90,
            pattern_retention_days: 365, // Archive old patterns after 1 year
            // Limits
            max_events: 10_000,
            max_patterns_per_type: 500,
            max_preferences_per_type: 50,
        }
    }

    pub fn with_custom_retention(
        event_days: i64,
        pattern_days: i64,
    ) -> Self {
        Self {
            event_retention_days: event_days,
            pattern_retention_days: pattern_days,
            max_events: 10_000,
            max_patterns_per_type: 500,
            max_preferences_per_type: 50,
        }
    }

    /// Get preview of what would be cleaned (for user authorization)
    pub async fn get_cleanup_preview(&self, db: &DatabaseConnection) -> Result<CleanupPreview, String> {
        let mut preview = CleanupPreview::default();

        // Count old events
        let events = LearningEventRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get events: {}", e))?;

        let cutoff = chrono::Utc::now() - chrono::Duration::days(self.event_retention_days);
        preview.events_to_delete = events.iter()
            .filter(|e| {
                if let Some(created_at) = &e.created_at {
                    // Compare timestamps
                    created_at.naive_utc() < cutoff.naive_utc()
                } else {
                    false
                }
            })
            .count() as u64;

        // Count low-quality patterns (excluding important ones)
        let patterns = LearnedPatternRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get patterns: {}", e))?;

        preview.patterns_to_delete = patterns.iter()
            .filter(|p| {
                !p.is_important && 
                ((p.success_rate < 0.2 && p.frequency < 3) ||
                 (p.frequency == 1 && p.success_rate < 0.5))
            })
            .count() as u64;

        // Count events over limit
        if events.len() > self.max_events {
            preview.events_over_limit = (events.len() - self.max_events) as u64;
        }

        Ok(preview)
    }

    /// Perform automatic cleanup of old data
    /// Only cleans non-important data by default
    pub async fn cleanup(&self, db: &DatabaseConnection) -> Result<CleanupStats, String> {
        let mut stats = CleanupStats::default();

        // Clean old events
        match LearningEventRepository::delete_older_than(db, self.event_retention_days).await {
            Ok(count) => {
                stats.events_deleted = count;
                log_info!("Learning", "Cleaned {} old events (older than {} days)", count, self.event_retention_days);
            }
            Err(e) => {
                log_warn!("Learning", "Failed to clean old events: {}", e);
            }
        }

        // Limit total events
        if let Err(e) = self.limit_events(db, self.max_events).await {
            log_warn!("Learning", "Failed to limit events: {}", e);
        }

        // Clean up low-quality patterns
        if let Err(e) = self.cleanup_low_quality_patterns(db).await {
            log_warn!("Learning", "Failed to clean low-quality patterns: {}", e);
        }

        // Consolidate similar patterns
        if let Err(e) = self.consolidate_patterns(db).await {
            log_warn!("Learning", "Failed to consolidate patterns: {}", e);
        }

        Ok(stats)
    }

    /// Limit total number of events, keeping most recent
    async fn limit_events(&self, db: &DatabaseConnection, max: usize) -> Result<(), String> {
        let recent = LearningEventRepository::get_recent(db, max as u64)
            .await
            .map_err(|e| format!("Failed to get recent events: {}", e))?;

        if recent.len() < max {
            return Ok(()); // Not at limit yet
        }

        // Get all events and find IDs to delete
        let all_events = LearningEventRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get all events: {}", e))?;

        let recent_ids: std::collections::HashSet<i32> = recent.iter()
            .map(|e| e.id)
            .collect();

        // Delete events not in recent set
        let mut deleted = 0;
        for event in all_events {
            if !recent_ids.contains(&event.id) {
                LearningEventRepository::delete(db, event.id)
                    .await
                    .map_err(|e| format!("Failed to delete event: {}", e))?;
                deleted += 1;
            }
        }

        if deleted > 0 {
            log_info!("Learning", "Limited events to {} most recent, deleted {} old events", max, deleted);
        }

        Ok(())
    }

    /// Clean up patterns with low success rate or low frequency
    /// Never deletes patterns marked as important
    async fn cleanup_low_quality_patterns(&self, db: &DatabaseConnection) -> Result<(), String> {
        // Get all patterns
        let patterns = LearnedPatternRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get patterns: {}", e))?;

        let mut deleted = 0;
        for pattern in patterns {
            // NEVER delete important patterns
            if pattern.is_important {
                continue;
            }

            // Delete patterns that are:
            // - Very low success rate (< 0.2) AND low frequency (< 3)
            // - Or patterns older than retention period with very low frequency
            let should_delete = (pattern.success_rate < 0.2 && pattern.frequency < 3)
                || (pattern.frequency == 1 && pattern.success_rate < 0.5);

            if should_delete {
                LearnedPatternRepository::delete(db, pattern.id)
                    .await
                    .map_err(|e| format!("Failed to delete pattern: {}", e))?;
                deleted += 1;
            }
        }

        if deleted > 0 {
            log_info!("Learning", "Cleaned {} low-quality patterns", deleted);
        }

        Ok(())
    }

    /// Consolidate similar patterns (merge duplicates)
    async fn consolidate_patterns(&self, db: &DatabaseConnection) -> Result<(), String> {
        // Group patterns by type and context
        let patterns = LearnedPatternRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get patterns: {}", e))?;

        use std::collections::HashMap;
        let mut pattern_groups: HashMap<(String, Option<String>), Vec<_>> = HashMap::new();

        for pattern in &patterns {
            let key = (pattern.pattern_type.clone(), pattern.context.clone());
            pattern_groups.entry(key).or_insert_with(Vec::new).push(pattern);
        }

        let mut consolidated = 0;
        for (_, group) in pattern_groups {
            if group.len() <= 1 {
                continue; // No duplicates to merge
            }

            // Find patterns with identical pattern_data
            let mut by_data: HashMap<&str, Vec<_>> = HashMap::new();
            for pattern in &group {
                by_data.entry(&pattern.pattern_data).or_insert_with(Vec::new).push(pattern);
            }

            // Merge duplicates (keep the one with highest frequency/success rate)
            for (_, duplicates) in by_data {
                if duplicates.len() <= 1 {
                    continue;
                }

                // Find the best pattern to keep (prefer important ones)
                let best = duplicates.iter()
                    .max_by(|a, b| {
                        // Important patterns always win
                        match (a.is_important, b.is_important) {
                            (true, false) => return std::cmp::Ordering::Greater,
                            (false, true) => return std::cmp::Ordering::Less,
                            _ => {}
                        }
                        // Compare by frequency, then success rate
                        a.frequency.cmp(&b.frequency)
                            .then_with(|| a.success_rate.partial_cmp(&b.success_rate).unwrap_or(std::cmp::Ordering::Equal))
                    });
                    
                let best = if let Some(best_pattern) = best {
                    best_pattern
                } else {
                    // No duplicates found, skip this group
                    continue;
                };

                // Merge others into best (but never delete important patterns)
                for pattern in &duplicates {
                    if pattern.id == best.id {
                        continue;
                    }

                    // Don't delete important patterns, even if they're duplicates
                    if pattern.is_important {
                        continue;
                    }

                    // Update best pattern with merged stats
                    let total_frequency = best.frequency + pattern.frequency;
                    // Calculate weighted average success rate for merged pattern
                    let _merged_success_rate = if total_frequency > 0 {
                        ((best.success_rate * best.frequency as f64) + 
                         (pattern.success_rate * pattern.frequency as f64)) / total_frequency as f64
                    } else {
                        best.success_rate
                    };
                    // Note: merged_success_rate will be recalculated when increment_frequency is called

                    // Delete duplicate
                    LearnedPatternRepository::delete(db, pattern.id)
                        .await
                        .map_err(|e| format!("Failed to delete duplicate pattern: {}", e))?;

                    // Update best pattern
                    if total_frequency != best.frequency {
                        LearnedPatternRepository::increment_frequency(db, best.id, true)
                            .await
                            .map_err(|e| format!("Failed to update merged pattern: {}", e))?;
                    }

                    consolidated += 1;
                }
            }
        }

        if consolidated > 0 {
            log_info!("Learning", "Consolidated {} duplicate patterns", consolidated);
        }

        Ok(())
    }

    /// Get memory usage statistics
    pub async fn get_stats(&self, db: &DatabaseConnection) -> Result<MemoryStats, String> {
        let events = LearningEventRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get events: {}", e))?;

        let patterns = LearnedPatternRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get patterns: {}", e))?;

        let preferences = UserPreferenceRepository::get_all(db)
            .await
            .map_err(|e| format!("Failed to get preferences: {}", e))?;

        Ok(MemoryStats {
            total_events: events.len(),
            total_patterns: patterns.len(),
            total_preferences: preferences.len(),
            events_retention_days: self.event_retention_days,
            patterns_retention_days: self.pattern_retention_days,
            max_events: self.max_events,
            max_patterns_per_type: self.max_patterns_per_type,
        })
    }
}

#[derive(Default, Debug)]
pub struct CleanupStats {
    pub events_deleted: u64,
    pub patterns_deleted: u64,
    pub patterns_consolidated: u64,
}

#[derive(Default, Debug)]
pub struct CleanupPreview {
    pub events_to_delete: u64,
    pub patterns_to_delete: u64,
    pub events_over_limit: u64,
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_events: usize,
    pub total_patterns: usize,
    pub total_preferences: usize,
    pub events_retention_days: i64,
    pub patterns_retention_days: i64,
    pub max_events: usize,
    pub max_patterns_per_type: usize,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

