mod cache;
mod pack;
mod repo;
mod useg;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use log::{Level, debug};
use pack::Packer;
use repo::{BlobKind, Index, IndexPackInfo, Key, Node, NodeKind, PackInfoEntry, Tree};
use useg::{UPath, USeg};
use walkdir::WalkDir;

/// Backup, now with reasonable performance.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// does testing things
    Backup {
        /// path to backup
        #[arg(short, long)]
        path: PathBuf,

        /// path to repo
        #[arg(short, long)]
        repo: PathBuf,
    },
}

fn main() {
    let args = Args::parse();
    env_logger::builder()
        .filter(None, Level::Debug.to_level_filter())
        .format_timestamp_millis()
        .init();

    debug!("and we're alive!");

    match args.command {
        Command::Backup { path, repo } => {
            run_backup(&path, &repo);
        }
    }
}

fn run_backup(path: &Path, repo: &Path) {
    let mut trees = Vec::new();
    let mut file_packer = Packer::new();
    let mut tree_packer = Packer::new();

    let data_path = repo.join("data");
    let tree_path = repo.join("tree");
    let index_path = repo.join("index");

    fs::create_dir_all(repo).unwrap();
    fs::create_dir_all(&data_path).unwrap();
    fs::create_dir_all(&tree_path).unwrap();
    fs::create_dir_all(&index_path).unwrap();

    let key = Key {
        mac: [0; 32],
        encrypt: [0; 32],
    };

    let mut index = Index {
        supersedes: Vec::new(),
        packs: Vec::new(),
    };

    for entry in WalkDir::new(path).sort_by_file_name() {
        let entry = entry.unwrap();
        let upath = UPath::from_path(entry.path());

        debug!("entry path: {:?}", entry.path());

        if entry.file_type().is_dir() {
            //let subtree = USeg::from_segment_bytes(upath.last_segment());
            //let kind = NodeKind::Dir { subtree };
            //add_node(&mut trees, kind, &upath);

            let tree = Tree {
                nodes: BTreeSet::new(),
            };

            trees.push((upath.clone(), tree));
        }

        if entry.file_type().is_file() {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(entry.path())
                .unwrap();

            let mut content = Vec::new();

            for (entry, chunk) in pack::split_to_data_blobs(&mut file) {
                content.push(entry.id);
                file_packer.add_blob(entry, &chunk);

                if file_packer.should_pack() {
                    finish_pack(&mut file_packer, &mut index, &key, &data_path);
                }
            }

            let kind = NodeKind::File { content };

            add_node(&mut trees, kind, &upath);
        }
    }

    for (_path, tree) in trees.into_iter().rev() {
        let data = serde_json::to_vec(&tree).unwrap();
        let id = blake3::hash(&data);
        let entry = PackInfoEntry {
            id,
            kind: BlobKind::Tree,
            size_uncompressed: data.len(),
            size_compressed: None,
        };

        tree_packer.add_blob(entry, &data);
        if tree_packer.should_pack() {
            finish_pack(&mut tree_packer, &mut index, &key, &tree_path);
        }
    }

    finish_pack(&mut file_packer, &mut index, &key, &data_path);
    finish_pack(&mut tree_packer, &mut index, &key, &tree_path);

    let index_data = serde_json::to_vec(&index).unwrap();
    let index_id = blake3::hash(&index_data);
    let index_path = index_path.join(format!("{}.index", index_id));
    fs::write(index_path, index_data).unwrap();
}

fn add_node(trees: &mut Vec<(UPath, Tree)>, kind: NodeKind, upath: &UPath) {
    let node = Node {
        name: USeg::from_segment_bytes(upath.last_segment()),
        mode: 0,
        mtime: 0,
        atime: 0,
        ctime: 0,
        uid: 0,
        gid: 0,
        user: String::new(),
        inode: 0,
        kind,
    };

    debug!("node: {:?}", node);

    let parent = upath.parent();
    let tree_idx = trees.iter().position(|(path, _)| path == &parent).unwrap();
    trees[tree_idx].1.nodes.insert(node);
}

fn finish_pack(packer: &mut Packer, index: &mut Index, key: &Key, to: &Path) {
    let (pack, data) = packer.finish(&key);
    let pack_path = to.join(format!("{}.pack", pack.id));

    debug!("writing pack {:?}", index);

    index.packs.push(pack);

    if fs::exists(&pack_path).unwrap() {
        panic!();
    }

    fs::write(pack_path, data).unwrap();
}
