use memmap2::MmapMut;
use rand::{RngCore, SeedableRng, rngs::SmallRng};
use rayon::prelude::*;
use semaphore_rs::group::{ELEMENT_SIZE, Group};
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

/// Generates a Semaphore LeanIMT with 2^exponent leaves.
/// This uses semaphore's Group module, which sets up a Poseidon based LeanIMT.
pub fn generate_tree(exponent: u32, output_path: &str) {
    let num_leaves = 2u32.pow(exponent);
    println!(
        "Generating tree with 2^{} = {} leaves",
        exponent, num_leaves
    );

    let available_threads = thread::available_parallelism().unwrap().get();
    let chunk_size = (1024 * 1024).min(num_leaves as usize / available_threads + 1);
    let num_chunks = (num_leaves as usize + chunk_size - 1) / chunk_size;
    println!(
        "Using {} threads with {} chunks",
        available_threads, num_chunks
    );

    let temp_file_path = format!("{}.temp", output_path);
    let file_size = num_leaves as usize * ELEMENT_SIZE;

    let file = File::create(&temp_file_path).unwrap();
    file.set_len(file_size as u64).unwrap();

    // Memory map the file
    let file = File::options()
        .read(true)
        .write(true)
        .open(&temp_file_path)
        .unwrap();
    let mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
    let mmap = Arc::new(Mutex::new(mmap));

    // Generate random elements
    (0..num_chunks).into_par_iter().for_each(|chunk_idx| {
        let mut rng = SmallRng::from_entropy();

        let start = chunk_idx * chunk_size;
        let end = std::cmp::min(start + chunk_size, num_leaves as usize);

        let mut local_buffer = Vec::with_capacity(32 * (end - start));

        // Generate random data for this chunk
        for _ in start..end {
            let mut element = [0u8; 32];
            rng.fill_bytes(&mut element);

            // Ensure no zero element
            if element.iter().all(|&b| b == 0) {
                element[0] = 1;
            }

            local_buffer.extend_from_slice(&element);
        }

        // Write to mmap
        let mut mmap = mmap.lock().unwrap();
        let slice_start = start * 32;
        let slice_end = slice_start + local_buffer.len();
        mmap[slice_start..slice_end].copy_from_slice(&local_buffer);

        // Progress updates
        if chunk_idx % 10 == 0 || chunk_idx == num_chunks - 1 {
            println!("Generated chunk {}/{}", chunk_idx + 1, num_chunks);
        }
    });

    // Build and save the tree
    println!("Building tree...");
    let tree = build_tree_from_mmap(&mmap.lock().unwrap(), num_leaves as usize);
    save_tree(&tree, output_path);

    // Clean up temp file
    drop(mmap);
    drop(file);
    std::fs::remove_file(temp_file_path).unwrap();

    println!("Tree generated and saved successfully to {}", output_path);
}

/// Builds a Semaphore tree from a memory-mapped file
fn build_tree_from_mmap(mmap: &MmapMut, num_leaves: usize) -> Group {
    const BATCH_SIZE: usize = 1_000_000; // 1M leaves per batch
    let mut elements = vec![[0u8; 32]; std::cmp::min(BATCH_SIZE, num_leaves)];
    let mut tree = None;

    for batch_idx in 0..(num_leaves + BATCH_SIZE - 1) / BATCH_SIZE {
        let batch_start = batch_idx * BATCH_SIZE;
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, num_leaves);
        let batch_size = batch_end - batch_start;

        // Read batch from mmap
        for i in 0..batch_size {
            let element_idx = batch_start + i;
            let byte_start = element_idx * 32;
            elements[i].copy_from_slice(&mmap[byte_start..byte_start + 32]);
        }

        // Create or update tree
        if tree.is_none() {
            tree = Some(Group::new(&elements[0..batch_size]).unwrap());
        } else if batch_size > 0 {
            let mut t = tree.take().unwrap();
            t.add_members(&elements[0..batch_size]).unwrap();
            tree = Some(t);
        }

        // Progress updates
        if batch_idx % 5 == 0 || batch_idx == (num_leaves + BATCH_SIZE - 1) / BATCH_SIZE - 1 {
            println!(
                "Processed batch {}/{}",
                batch_idx + 1,
                (num_leaves + BATCH_SIZE - 1) / BATCH_SIZE
            );
        }
    }

    tree.unwrap()
}

/// Saves the tree to a JSON file
fn save_tree(tree: &Group, output_path: &str) {
    println!("Saving tree to {}", output_path);

    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }

    let num_leaves = tree.size();
    let depth = tree.depth();
    let total_elements = 2 * num_leaves - 1;

    let json = tree.export().unwrap();
    let file_size_bytes = json.as_bytes().len();
    let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);

    println!("Tree stats:");
    println!("Leaves: {}", num_leaves);
    println!("Tree depth: {}", depth);
    println!("Total elements: {}", total_elements);
    println!(
        "File size: {:.2} MB ({} bytes)",
        file_size_mb, file_size_bytes
    );

    let mut file = File::create(output_path).unwrap();
    file.write_all(json.as_bytes()).unwrap();

    println!("Tree saved successfully");
}

/// CLI wrapper
pub fn generate_and_save_tree(exponent: u32) {
    // Standardized output path
    let output_path = format!("trees/tree_{}.json", exponent);

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("trees").unwrap_or_else(|err| {
        eprintln!("Error creating trees directory: {}", err);
        std::process::exit(1);
    });

    // Generate the tree
    generate_tree(exponent, &output_path);

    println!(
        "Tree with 2^{} leaves generated at {}",
        exponent, output_path
    );
}
