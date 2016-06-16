extern crate taglib;
extern crate walkdir;

use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process;

use walkdir::WalkDir;

/*====================================================================================================*/
/*------STRUCTS---------------------------------------------------------------------------------------*/
/*====================================================================================================*/

// The music file struct
struct MusicFile {

    pub artist      : String,
    pub album       : String,
    pub title       : String,
    pub format      : String,
    pub source_file : String,
    pub dest_file   : String
}

/*====================================================================================================*/

// The music options struct
struct MusicItems {

    pub source_folder   : String,
    pub dest_folder     : String,

    pub music_files_found   : i32,
    pub music_files_copied  : i32,
    pub music_files_skipped : i32,
    pub music_files_failed  : i32,

    pub file_list : Vec <MusicFile>
}

/*====================================================================================================*/
/*------FUNCTIONS-------------------------------------------------------------------------------------*/
/*====================================================================================================*/

// Check if directory exists
fn directory_exists (directory : &String) -> bool {

    match fs::metadata (directory) {

        Ok (meta) => {

            if meta.is_file () {
                return false;
            }

            else if meta.file_type ().is_symlink () {
                return false;
            }

            true
        },

        Err (_) => false
    }
}

/*====================================================================================================*/

// Check if file exists
fn file_exists (file : &String) -> bool {

    match fs::metadata (file) {

        Ok (meta) => {

            if meta.is_dir () {
                return false;
            }

            else if meta.file_type ().is_symlink () {
                return false;
            }

            true
        },

        Err (_) => false
    }
}

/*====================================================================================================*/

// Set the source folder
fn set_source_folder () -> String {

    // Set variables
    let mut source_folder = String::new ();

    // Prompt user for music folder
    print! ("\nEnter the music folder : ");

    // Flush the IO stream, and read user input
    stdout  ().flush ().unwrap ();
    stdin   ().read_line (&mut source_folder).unwrap ();
    source_folder = source_folder.trim ().to_string ();

    // Check if the directory exists
    if !directory_exists (&source_folder) {

        println! ("\nERROR: Invalid directory\nPlease try again\n");
        return set_source_folder ();
    }

    // Return the source folder
    source_folder
}

/*====================================================================================================*/

// Set the destination folder
fn set_dest_folder () -> String {

    // Set variables
    let mut dest_folder = String::new ();

    // Prompt user for destination folder
    print! ("Enter the destination folder : ");

    // Flush the IO stream and read user input
    stdout  ().flush ().unwrap ();
    stdin   ().read_line (&mut dest_folder).unwrap ();
    dest_folder = dest_folder.trim ().to_string ();

    // Check if directory exists
    if !directory_exists (&dest_folder) {

        // Set input variable
        let mut ans = String::new ();

        // Ask if user want to create new folder
        print! ("\nDirectory does not exist\nWould you like to create it (y/n)? : ");

        // Flush the IO stream and read user input
        stdout  ().flush ().unwrap ();
        stdin   ().read_line (&mut ans).unwrap ();
        ans = ans.trim ().to_string ();

        // If yes, create new folder
        if ans == "y" {

            let mut new_dir = fs::DirBuilder::new ();
            new_dir.recursive   (true);
            new_dir.create      (&dest_folder).unwrap ();
        }

        // Otherwise, abort
        else {

            println! ("\nERROR: Destination folder does not exist\nCannot continue\n\nAborting...");
            process::exit (0);
        }
    }

    println! ("");

    // Return the source folder
    dest_folder
}

/*====================================================================================================*/

// Scan for music files
fn scan_for_music_files (music_items : &mut MusicItems) {

    println! ("");

    // Loop through all files and directories in the source folder
    for file in WalkDir::new (&music_items.source_folder) {

        let file_path = file.unwrap ().path ().display ().to_string ();

        match fs::metadata (&file_path) {

            // Check if ok
            Ok (meta) => {

                // Check if audio file
                if meta.is_file () && is_audio_file (&file_path) {

                    // Create a new musicfile instance, and add file path
                    music_items.file_list.push (MusicFile {artist       : String::new (),
                                                           album        : String::new (),
                                                           title        : String::new (),
                                                           format       : String::new (),
                                                           source_file  : file_path,
                                                           dest_file    : String::new ()});

                    // Increment file count
                    music_items.music_files_found += 1;

                    print! ("Scanning for music files (found {})...\r", music_items.music_files_found);
                    stdout ().flush ().unwrap ();
                }
            },

            // Else throw error
            Err (_) => {

                //println! ("\nError while searching for music files : {}\n\nAborting...", error);
                //process::exit (0);
            }
        }
    }

    println! ("Scanning for music files (found {})... Complete!", music_items.music_files_found);
}

/*====================================================================================================*/

// Check if audio file
fn is_audio_file (file : &String) -> bool {

    match taglib::File::new (&file) {

        Ok  (_) => true,
        Err (_) => false
    }
}

/*====================================================================================================*/

// Extract all metadata
fn extract_metadata (music_items : &mut MusicItems) {

    let mut counter : i64 = 1;

    // Loop through all files
    for file in &mut music_items.file_list {

        //print! ("---File {} of {}\r", counter, music_items.music_files_found);
        print! ("Extracting metadata ({} of {})...\r", counter - 1, music_items.music_files_found);
        stdout  ().flush ().unwrap ();

        let mut file_title_unknown = false;

        // Get all other metadata
        match taglib::File::new (&file.source_file) {

            Ok (context) => {

                // Get tags from file
                match context.tag () {

                    Ok (t) => {

                        // Artist
                        if t.artist () != "" {
                            file.artist = t.artist ();
                        }

                        else {
                            file.artist = "Unknown".to_string ()
                        }

                        // Album
                        if t.album () != "" {
                            file.album = t.album ();
                        }

                        else {
                            file.album = "Unknown".to_string ()
                        }

                        // Title
                        if t.title () != "" {
                            file.title = t.title ();
                        }

                        else {

                            file_title_unknown = true;

                            match Path::new (&file.source_file).file_name () {

                                Some (x) => {file.title = x.to_str ().unwrap ().to_string ()},
                                None => {}
                            }
                        }
                    },

                    // If no tags, set as unknown
                    Err (_) => {

                        file.artist = "Unknown".to_string ();
                        file.album  = "Unknown".to_string ();

                        file_title_unknown = true;

                        match Path::new (&file.source_file).file_name () {

                            Some (x) => {file.title = x.to_str ().unwrap ().to_string ()},
                            None => {}
                        }
                    }
                }
            },

            Err (_) => {}
        }

        if !file_title_unknown {

            // Get file format
            match Path::new (&file.source_file).extension () {

                Some (x) => {file.format = ".".to_string () + &*x.to_str ().unwrap ().to_string ()},
                None => {}
            }
        }

        // Set destination path
        file.dest_file = music_items.dest_folder.clone () + "/" +
                         &file.artist + "/" +
                         &file.album + "/" +
                         &file.title +
                         &file.format;

        // Increment counter
        counter += 1;
    }

    println! ("Extracting metadata ({} of {})... Complete!", counter - 1, music_items.music_files_found);
}

/*====================================================================================================*/

// Copy all the music files
fn copy_music_files (music_items : &mut MusicItems) {

    let mut counter : i64 = 1;

    for file in &music_items.file_list {

        print! ("Copying files ({} of {})...\r", counter - 1, music_items.music_files_found);
        stdout  ().flush ().unwrap ();

        // Check if dest folder exists. If not create it
        match Path::new (&file.dest_file).parent () {

            Some (x) => {

                if !directory_exists (&x.to_str ().unwrap ().to_string ()) {

                    let mut new_dir = fs::DirBuilder::new ();
                    new_dir.recursive   (true);
                    new_dir.create      (&x.to_str ().unwrap ().to_string ()).unwrap ();
                }
            },

            None => {}
        }

        // Check if song already exists. If not copy it
        if !file_exists (&file.dest_file) {

            match fs::copy (&file.source_file, &file.dest_file) {

                Ok (_) => {music_items.music_files_copied += 1},

                Err (_) => {

                    //println! ("Failed to copy file ({}) : {}", file.source_file, error.to_string ());
                    //process::exit (-1);
                    music_items.music_files_failed += 1;
                }
            }
        }

        else {
            music_items.music_files_skipped += 1;
        }

        counter += 1;
    }

    println! ("Copying files ({} of {})... Complete!", counter - 1, music_items.music_files_found);
}

/*====================================================================================================*/

// Print the copy results
fn print_results (music_items: &MusicItems) {

    println! ("\n\nSorting results\n");
    println! ("    Files copied    : {}", music_items.music_files_copied);
    println! ("    Files skipped   : {}", music_items.music_files_skipped);
    println! ("    Files failed    : {}", music_items.music_files_failed);
}

/*====================================================================================================*/

// Main
fn main () {

    // Create a new instance of the music items
    let mut music_items = MusicItems {source_folder         : String::new (),
                                      dest_folder           : String::new (),
                                      music_files_found     : 0,
                                      music_files_copied    : 0,
                                      music_files_skipped   : 0,
                                      music_files_failed    : 0,
                                      file_list             : Vec::new ()};

    // Get the source and destination directories
    music_items.source_folder   = set_source_folder ();
    music_items.dest_folder     = set_dest_folder ();

    // Scan for music files, extract metadata, and copy files
    scan_for_music_files    (&mut music_items);
    extract_metadata        (&mut music_items);
    copy_music_files        (&mut music_items);

    print_results (&music_items);
}
