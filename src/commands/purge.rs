use crate::infra::{trash_store::TrashStoreInterface, ConfigManager, TrashStore};
use crate::utils::size_parser::parse_size;
use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Execute purge command
pub fn execute(
    all: bool,
    days: Option<u32>,
    size: Option<String>,
    id: Option<String>,
    interactive: bool,
    verbose: bool,
) -> Result<()> {
    let config = ConfigManager::load()?;
    let trash_store = TrashStore::new(config.trash_path.clone());

    // Determine candidate items
    let mut candidates = if let Some(id_str) = id {
        let id = Uuid::parse_str(&id_str).map_err(|_| anyhow::anyhow!("Invalid UUID format"))?;
        match trash_store.find_by_id(&id)? {
            Some(item) => vec![item],
            None => {
                println!("File with ID {} not found in trash", id);
                return Ok(());
            }
        }
    } else {
        trash_store.list()?
    };

    if !all {
        if let Some(days) = days {
            let cutoff = Utc::now() - Duration::days(days as i64);
            candidates.retain(|item| item.meta.deleted_at <= cutoff);
        }

        if let Some(size_str) = size {
            let limit = parse_size(&size_str)?;
            let mut all_items = candidates;
            let mut total: u64 = all_items.iter().map(|i| i.meta.size).sum();
            if total <= limit {
                println!("Current trash size below specified limit");
                return Ok(());
            }
            all_items.sort_by_key(|i| i.meta.deleted_at);
            let mut selected = Vec::new();
            for item in all_items {
                if total <= limit {
                    break;
                }
                total -= item.meta.size;
                selected.push(item);
            }
            candidates = selected;
        }
    }

    if candidates.is_empty() {
        println!("No files match the purge criteria");
        return Ok(());
    }

    if interactive {
        for item in &candidates {
            if !confirm_single(item)? {
                continue;
            }
            trash_store.purge(&item.meta.id)?;
            if verbose {
                println!("Purged: {}", item.meta.filename().unwrap_or("unknown"));
            }
        }
    } else {
        if !confirm_batch(candidates.len())? {
            println!("Purge operation cancelled");
            return Ok(());
        }
        for item in &candidates {
            trash_store.purge(&item.meta.id)?;
            if verbose {
                println!("Purged: {}", item.meta.filename().unwrap_or("unknown"));
            }
        }
    }

    Ok(())
}

fn confirm_single(item: &crate::domain::TrashItem) -> Result<bool> {
    use dialoguer::Confirm;
    let prompt = format!(
        "Permanently delete {}?",
        item.meta.filename().unwrap_or("unknown")
    );
    Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .map_err(|e| e.into())
}

fn confirm_batch(count: usize) -> Result<bool> {
    use dialoguer::Confirm;
    let prompt = format!("Permanently delete {} file(s)?", count);
    Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .map_err(|e| e.into())
}
