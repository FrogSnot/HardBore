use hardbore_lib::fs_engine::{crawl_directory, read_directory};
use hardbore_lib::indexer::Indexer;
use std::time::Instant;
use std::fs;

fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn main() {
    println!("Stress test :)\n");

    println!("Test 1: Single Directory Read (/usr/bin)");
    
    let start = Instant::now();
    let result = read_directory("/usr/bin", false);
    let read_time = start.elapsed();
    
    match result {
        Ok(contents) => {
            println!("  Files: {}", contents.total_items);
            println!("  Total Size: {}", format_size(contents.total_size));
            println!("  Time: {}", format_duration(read_time.as_millis()));
            println!("  Throughput: {:.0} items/sec\n", 
                contents.total_items as f64 / read_time.as_secs_f64());
        }
        Err(e) => println!("  Error: {}\n", e),
    }

    println!("Test 2: Recursive Crawl (/usr - max depth 3)");
    
    let start = Instant::now();
    let entries = crawl_directory("/usr", Some(3));
    let crawl_time = start.elapsed();
    
    let total_size: u64 = entries.iter().map(|e| e.size).sum();
    println!("  Files: {}", entries.len());
    println!("  Total Size: {}", format_size(total_size));
    println!("  Time: {}", format_duration(crawl_time.as_millis()));
    println!("  Throughput: {:.0} items/sec\n", 
        entries.len() as f64 / crawl_time.as_secs_f64());

    println!("Test 3: SQLite FTS5 Indexing");
    
    let temp_dir = std::env::temp_dir().join("hardbore_bench");
    fs::create_dir_all(&temp_dir).ok();
    let db_path = temp_dir.to_str().unwrap();
    
    let indexer = Indexer::new(db_path).expect("Failed to create indexer");
    
    println!("Indexing: /usr/share (depth 3)");
    
    let start = Instant::now();
    indexer.index_directory("/usr/share", Some(3));
    
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let status = indexer.get_status();
        if !status.is_running {
            break;
        }
    }
    let index_time = start.elapsed();
    
    let status = indexer.get_status();
    println!("  Indexed: {} files", status.indexed_count);
    println!("  Time: {}", format_duration(index_time.as_millis()));
    println!("  Rate: {:.0} files/sec\n", 
        status.indexed_count as f64 / index_time.as_secs_f64());

    println!("Test 4: FTS5 Search Performance");
    
    let queries = vec!["config", "test", "main", "lib", "README"];
    let mut total_results = 0;
    let mut total_time = 0u128;
    
    for query in &queries {
        let start = Instant::now();
        let results = indexer.search_fts(query, 100);
        let search_time = start.elapsed().as_micros();
        total_results += results.len();
        total_time += search_time;
        
        println!("  '{}': {} results in {}μs", 
            query, results.len(), search_time);
    }
    
    let avg_time = total_time / queries.len() as u128;
    println!("  Average: {}μs per query", avg_time);
    println!("  Total results: {}\n", total_results);

    println!("Test 5: Fuzzy Search Performance");
    
    let start = Instant::now();
    let results = indexer.search_fuzzy("conf", 50);
    let fuzzy_time = start.elapsed();
    
    println!("  Query: 'conf'");
    println!("  Results: {}", results.len());
    println!("  Time: {}", format_duration(fuzzy_time.as_millis()));
    println!();

    println!("Test 6: Large Directory Stress Test (/usr/lib - depth 2)");
    
    let start = Instant::now();
    let entries = crawl_directory("/usr/lib", Some(2));
    let crawl_time = start.elapsed();
    
    let total_size: u64 = entries.iter().map(|e| e.size).sum();
    println!("  Files: {}", entries.len());
    println!("  Total Size: {}", format_size(total_size));
    println!("  Time: {}", format_duration(crawl_time.as_millis()));
    println!("  Throughput: {:.0} items/sec\n", 
        entries.len() as f64 / crawl_time.as_secs_f64());

    println!("Test 7: Cold Start (DB open + search)");
    
    let start = Instant::now();
    let indexer2 = Indexer::new(db_path).expect("Failed");
    let results = indexer2.search_fts("test", 50);
    let cold_time = start.elapsed();
    
    println!("  Results: {}", results.len());
    println!("  Time: {}", format_duration(cold_time.as_millis()));
    println!();

    fs::remove_dir_all(temp_dir).ok();
    
    println!("Done ;)");
}

