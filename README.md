# rmz

> **Don’t worry, I’m wearing `rmz`.**  
> A safe, reversible layer for every file you remove.

`rmz` is a safer alternative to `rm`.  
Instead of permanently deleting your files, it moves them to a hidden zone (`.rmz/`),  
where they remain recoverable — until you decide otherwise.

So go ahead.  
Delete with confidence.  
Nothing’s truly gone, just safely stored away.

---

## Features

- 🛡️ `rm`-compatible CLI — replace `rm` without changing your habits
- ♻️ Safe file removal — files are moved, not destroyed
- 🔍 `rmz list` — browse past deletions grouped by operation
- 🌲 `--tree` view — see deleted files in their original structure
- 🧪 `--dry-run` — preview restoration conflicts, directories to be created
- ✅ UUID-based tracking — accurate, collision-free identification
- 💥 `rmz purge` — permanently erase when *you* decide to

---

## Philosophy

> It’s not just `rm`. It’s `rm`, with a conscience.

### Why the “z”?

The **z** in `rmz` stands for:

- **Zone** – a shadow space where deleted files are safely held
- **Zero-impact** – your deletions won’t destroy, just detach
- **Zenith** – the final evolution of `rm`: safety-first and user-forgiving

It’s a shell, a safeguard, a second chance.

---

## Example

```bash
# Remove a file
$ rmz delete main.rs

# List deleted operations
$ rmz list

# View the structure of a deletion
$ rmz list --tree 1a2b3c4d

# Dry-run restore to see what would happen
$ rmz restore 1a2b3c4d --dry-run

# Restore the operation
$ rmz restore 1a2b3c4d


⸻

Why use rmz?
	•	You deleted the wrong file.
	•	You thought you didn’t need it — but you did.
	•	You’re human.

rmz gives you what rm never could:
a way back.

⸻

License

MIT
