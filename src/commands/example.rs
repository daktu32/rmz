use colored::*;

pub fn execute(verbose: bool) -> anyhow::Result<()> {
    if verbose {
        println!("{}", "Running in verbose mode".green());
    }
    
    println!("{}: {}", "rmz".blue().bold(), "The next gen rm command");
    println!("Example command executed successfully!");
    
    Ok(())
}