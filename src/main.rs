use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::env;
use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

fn main() -> Result<()> {
    // Get the download URL from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} DOWNLOAD_URL", args[0]);
        std::process::exit(1);
    }
    let download_url = &args[1];

    // Create a "temp" directory in the current directory
    let temp_dir_path = Path::new("temp");
    if temp_dir_path.exists() {
        fs::remove_dir_all(temp_dir_path).context("Failed to remove existing temp directory")?;
        println!("Removed existing temp directory");
    }
    fs::create_dir(temp_dir_path).context("Failed to create temp directory")?;
    println!("Created directory at: {}", temp_dir_path.display());

    // Download the archive
    println!("Downloading from: {}", download_url);
    let response = reqwest::blocking::get(download_url).context("Failed to download file")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download file: HTTP status {}", response.status());
    }

    // Create a temporary file to store the archive
    let archive_path = temp_dir_path.join("archive.tar.gz");
    let mut archive_file =
        File::create(&archive_path).context("Failed to create temporary archive file")?;

    // Save the downloaded content to the temporary file
    copy(
        &mut response
            .bytes()
            .context("Failed to read response")?
            .as_ref(),
        &mut archive_file,
    )
    .context("Failed to save archive")?;

    // Extract the archive
    println!("Extracting archive to: {}", temp_dir_path.display());
    extract_tar_gz(&archive_path, temp_dir_path).context("Failed to extract archive")?;

    // Move contents from top-level subfolder to temp directory
    let entries = fs::read_dir(temp_dir_path)
        .context("Failed to read temp directory")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect directory entries")?;

    // Find directories (excluding the archive file)
    let subfolders: Vec<_> = entries
        .iter()
        .filter(|entry| {
            let path = entry.path();
            path.is_dir() && path.file_name() != Some(archive_path.file_name().unwrap_or_default())
        })
        .collect();

    // If there's a single subfolder, move its contents up
    if subfolders.len() == 1 {
        let subfolder_path = &subfolders[0].path();
        println!(
            "Moving contents from subfolder: {}",
            subfolder_path.display()
        );

        // Move all contents from subfolder to temp directory
        let subfolder_entries = fs::read_dir(subfolder_path).context("Failed to read subfolder")?;

        for entry in subfolder_entries {
            let entry = entry.context("Failed to read subfolder entry")?;
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let target_path = temp_dir_path.join(file_name);

            // Move the file/directory
            fs::rename(&source_path, &target_path).context(format!(
                "Failed to move {:?} to {:?}",
                source_path, target_path
            ))?;
        }

        // Remove the empty subfolder
        fs::remove_dir(subfolder_path).context("Failed to remove empty subfolder")?;
        println!("Successfully moved contents and removed subfolder");
    }

    println!("Archive successfully extracted");

    // cd into the temp directory
    std::env::set_current_dir(temp_dir_path).context("Failed to set current directory")?;

    // Detect build system based on presence of build files
    let maven_patterns = [
        "pom.xml",
        "pom.atom",
        "pom.clj",
        "pom.groovy",
        "pom.rb",
        "pom.scala",
        "pom.yaml",
        "pom.yml",
    ];

    let is_maven = maven_patterns
        .iter()
        .any(|pattern| Path::new(pattern).exists());

    let is_gradle = Path::new("gradlew").exists();

    if is_maven {
        println!("Using Maven");
        // print the maven version by running "mvn version"
        let output = Command::new("mvn")
            .arg("--version")
            .output()
            .context("Failed to run mvn version")?;
        println!("Maven version: {}", String::from_utf8_lossy(&output.stdout));

        // run "mvn clean package -Dmaven.test.skip=true"
        let output = Command::new("mvn")
            .args(&["clean", "package", "-Dmaven.test.skip=true"])
            .output()
            .context("Failed to run mvn clean package")?;

        // print the output
        println!(
            "Maven build output:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else if is_gradle {
        println!("Using Gradle");

        // run "./gradlew clean build -x check -x test"
        let output = Command::new("./gradlew")
            .args(&["clean", "build", "-x", "check", "-x", "test"])
            .output()
            .context("Failed to run gradlew")?;

        // print the output
        println!(
            "Gradle build output:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        anyhow::bail!(
            "No build system detected. Make sure your project contains a pom.xml/pom.groovy/... or gradlew file. If you're using Gradle but there is no gradlew file, run 'gradle wrapper' to generate one."
        );
    }

    Ok(())
}

fn extract_tar_gz(archive_path: &PathBuf, dest_path: &Path) -> Result<()> {
    let tar_gz = File::open(archive_path).context("Failed to open archive file")?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive
        .unpack(dest_path)
        .context("Failed to unpack archive")?;
    Ok(())
}
